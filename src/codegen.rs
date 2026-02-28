use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::support::LLVMString;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::values::{
    BasicMetadataValueEnum, FloatValue, FunctionValue,
    IntValue, PointerValue,
};

use crate::ast::*;

// ── Variable kind ──────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
enum VarKind {
    Int,
    Float,
    /// `ptr` is a pointer directly to the struct data in memory.
    Struct(String),
}

// ── Variable descriptor ────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct Var<'ctx> {
    ptr:  PointerValue<'ctx>,
    kind: VarKind,
}

// ── Typed runtime value ────────────────────────────────────────────────────

#[derive(Clone, Debug)]
enum Val<'ctx> {
    Int(IntValue<'ctx>),
    Float(FloatValue<'ctx>),
    /// Struct value is always represented as a pointer to its stack allocation.
    Struct(PointerValue<'ctx>, String),
}

impl<'ctx> Val<'ctx> {
    fn is_float(&self) -> bool { matches!(self, Val::Float(_)) }
}

// ── Code generator ──────────────────────────────────────────────────────────

pub struct CodeGen<'ctx> {
    ctx:     &'ctx Context,
    builder: Builder<'ctx>,
    module:  Module<'ctx>,
    vars:    HashMap<String, Var<'ctx>>,
    i64_ty:  inkwell::types::IntType<'ctx>,
    f64_ty:  inkwell::types::FloatType<'ctx>,
    /// LLVM struct types keyed by struct name.
    struct_types:  HashMap<String, inkwell::types::StructType<'ctx>>,
    /// Ordered field info (name, is_float) keyed by struct name.
    struct_fields: HashMap<String, Vec<(String, bool)>>,
    /// Stack of (exit_bb, continue_bb) for the active loop nesting.
    loop_stack: Vec<(BasicBlock<'ctx>, BasicBlock<'ctx>)>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(module_name: &str, ctx: &'ctx Context) -> Self {
        let module  = ctx.create_module(module_name);
        let builder = ctx.create_builder();
        let i64_ty  = ctx.i64_type();
        let f64_ty  = ctx.f64_type();

        // Declare libc printf: int printf(char*, ...)
        let i8_ptr    = ctx.ptr_type(inkwell::AddressSpace::default());
        let printf_ty = ctx.i32_type().fn_type(&[i8_ptr.into()], true);
        module.add_function("printf", printf_ty, None);

        Self {
            ctx,
            builder,
            module,
            vars:          HashMap::new(),
            i64_ty,
            f64_ty,
            struct_types:  HashMap::new(),
            struct_fields: HashMap::new(),
            loop_stack:    Vec::new(),
        }
    }

    // ── Program ──────────────────────────────────────────────────────────

    pub fn generate_program(&mut self, program: &[Stmt]) {
        // Pass 0: collect struct type declarations.
        for stmt in program {
            if let Stmt::StructDecl { name, fields } = stmt {
                self.declare_struct(name, fields);
            }
        }

        // Pass 1: forward-declare functions and methods.
        for stmt in program {
            match stmt {
                Stmt::FnDecl { name, params, .. } => {
                    let ptys: Vec<BasicMetadataTypeEnum> =
                        params.iter().map(|_| self.i64_ty.into()).collect();
                    self.module.add_function(
                        name,
                        self.i64_ty.fn_type(&ptys, false),
                        None,
                    );
                }
                Stmt::ImplDecl { struct_name, methods } => {
                    for m in methods {
                        self.forward_declare_method(struct_name, m);
                    }
                }
                _ => {}
            }
        }

        // Pass 2: generate function / method bodies.
        for stmt in program {
            match stmt {
                Stmt::FnDecl { name, params, body } => {
                    self.gen_fn(name, params, body);
                }
                Stmt::ImplDecl { struct_name, methods } => {
                    for m in methods {
                        self.gen_method(struct_name, m);
                    }
                }
                Stmt::StructDecl { .. } => {} // already handled in pass 0
                s => panic!("Unexpected top-level statement: {:?}", s),
            }
        }
    }

    // ── Struct helpers ────────────────────────────────────────────────────

    fn declare_struct(&mut self, name: &str, fields: &[(String, FieldType)]) {
        let field_types: Vec<inkwell::types::BasicTypeEnum> = fields
            .iter()
            .map(|(_, ft)| match ft {
                FieldType::Int      => self.i64_ty.into(),
                FieldType::Float    => self.f64_ty.into(),
                FieldType::Named(_) => self.i64_ty.into(), // future: nested structs
            })
            .collect();
        let st = self.ctx.struct_type(&field_types, false);
        self.struct_types.insert(name.to_string(), st);
        let info: Vec<(String, bool)> = fields
            .iter()
            .map(|(n, ft)| (n.clone(), matches!(ft, FieldType::Float)))
            .collect();
        self.struct_fields.insert(name.to_string(), info);
    }

    fn forward_declare_method(&mut self, struct_name: &str, method: &MethodDecl) {
        let func_name = format!("{}_{}", struct_name, method.name);
        let ptr_ty    = self.ctx.ptr_type(inkwell::AddressSpace::default());
        let mut ptys: Vec<BasicMetadataTypeEnum> = Vec::new();
        if method.has_self {
            ptys.push(ptr_ty.into());
        }
        ptys.extend(method.params.iter().map(|_| BasicMetadataTypeEnum::from(self.i64_ty)));
        self.module.add_function(
            &func_name,
            self.i64_ty.fn_type(&ptys, false),
            None,
        );
    }

    // ── Function ─────────────────────────────────────────────────────────

    fn gen_fn(&mut self, name: &str, params: &[String], body: &[Stmt]) {
        let func  = self.module.get_function(name)
            .unwrap_or_else(|| panic!("BUG: function '{}' not pre-declared", name));

        let entry = self.ctx.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);

        let outer_vars = std::mem::take(&mut self.vars);

        for (i, pname) in params.iter().enumerate() {
            let alloca = self.builder.build_alloca(self.i64_ty, pname).unwrap();
            let pval   = func.get_nth_param(i as u32).unwrap().into_int_value();
            self.builder.build_store(alloca, pval).unwrap();
            self.vars.insert(pname.clone(), Var { ptr: alloca, kind: VarKind::Int });
        }

        for s in body {
            if self.terminated() { break; }
            self.gen_stmt(s);
        }

        if !self.terminated() {
            self.builder
                .build_return(Some(&self.i64_ty.const_int(0, false)))
                .unwrap();
        }

        self.vars = outer_vars;
    }

    // ── Method ────────────────────────────────────────────────────────────

    fn gen_method(&mut self, struct_name: &str, method: &MethodDecl) {
        let func_name = format!("{}_{}", struct_name, method.name);
        let func = self.module.get_function(&func_name)
            .unwrap_or_else(|| panic!("BUG: method '{}' not pre-declared", func_name));

        let entry = self.ctx.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);

        let outer_vars = std::mem::take(&mut self.vars);

        let mut param_idx = 0u32;

        if method.has_self {
            // `self` is passed as a raw pointer to the struct data.
            let self_ptr = func.get_nth_param(0).unwrap().into_pointer_value();
            self.vars.insert(
                "self".into(),
                Var { ptr: self_ptr, kind: VarKind::Struct(struct_name.to_string()) },
            );
            param_idx = 1;
        }

        for (i, pname) in method.params.iter().enumerate() {
            let pval   = func.get_nth_param(param_idx + i as u32).unwrap().into_int_value();
            let alloca = self.builder.build_alloca(self.i64_ty, pname).unwrap();
            self.builder.build_store(alloca, pval).unwrap();
            self.vars.insert(pname.clone(), Var { ptr: alloca, kind: VarKind::Int });
        }

        for s in method.body.clone().iter() {
            if self.terminated() { break; }
            self.gen_stmt(s);
        }

        if !self.terminated() {
            self.builder
                .build_return(Some(&self.i64_ty.const_int(0, false)))
                .unwrap();
        }

        self.vars = outer_vars;
    }

    // ── Helpers ──────────────────────────────────────────────────────────

    fn terminated(&self) -> bool {
        self.builder
            .get_insert_block()
            .and_then(|b| b.get_terminator())
            .is_some()
    }

    fn cur_fn(&self) -> FunctionValue<'ctx> {
        self.builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap()
    }

    fn as_int(&self, v: Val<'ctx>) -> IntValue<'ctx> {
        match v {
            Val::Int(i)   => i,
            Val::Float(f) => self.builder
                .build_float_to_signed_int(f, self.i64_ty, "f2i")
                .unwrap(),
            Val::Struct(_, n) => panic!("Cannot coerce struct '{}' to integer", n),
        }
    }

    fn as_float(&self, v: Val<'ctx>) -> FloatValue<'ctx> {
        match v {
            Val::Float(f) => f,
            Val::Int(i)   => self.builder
                .build_signed_int_to_float(i, self.f64_ty, "i2f")
                .unwrap(),
            Val::Struct(_, n) => panic!("Cannot coerce struct '{}' to float", n),
        }
    }

    // ── Statements ───────────────────────────────────────────────────────

    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {

            // let name = expr;
            Stmt::Let { name, expr } => {
                let val = self.gen_expr(expr);
                match val {
                    Val::Int(i) => {
                        let p = self.builder.build_alloca(self.i64_ty, name).unwrap();
                        self.builder.build_store(p, i).unwrap();
                        self.vars.insert(name.clone(), Var { ptr: p, kind: VarKind::Int });
                    }
                    Val::Float(f) => {
                        let p = self.builder.build_alloca(self.f64_ty, name).unwrap();
                        self.builder.build_store(p, f).unwrap();
                        self.vars.insert(name.clone(), Var { ptr: p, kind: VarKind::Float });
                    }
                    Val::Struct(ptr, type_name) => {
                        // The struct already has its own alloca; just record the binding.
                        self.vars.insert(name.clone(), Var { ptr, kind: VarKind::Struct(type_name) });
                    }
                }
            }

            // name = expr;
            Stmt::Assign { name, expr } => {
                let var = self.vars.get(name)
                    .cloned()
                    .unwrap_or_else(|| panic!("Undefined variable '{}'", name));
                let val = self.gen_expr(expr);
                match var.kind {
                    VarKind::Float      => {
                        self.builder.build_store(var.ptr, self.as_float(val)).unwrap();
                    }
                    VarKind::Int        => {
                        self.builder.build_store(var.ptr, self.as_int(val)).unwrap();
                    }
                    VarKind::Struct(_)  => {
                        panic!("Direct struct re-binding via '=' is not supported yet");
                    }
                }
            }

            // obj.field = expr;
            Stmt::FieldAssign { obj, field, val } => {
                let obj_val = self.gen_expr(obj);
                if let Val::Struct(ptr, ref type_name) = obj_val {
                    let type_name = type_name.clone();
                    let field_info = self.struct_fields.get(&type_name)
                        .cloned()
                        .unwrap_or_else(|| panic!("Unknown struct '{}'", type_name));
                    let idx = field_info.iter().position(|(n, _)| n == field)
                        .unwrap_or_else(|| panic!("Unknown field '{}' on '{}'", field, type_name));
                    let (_, is_float) = field_info[idx];
                    let st = *self.struct_types.get(&type_name)
                        .unwrap_or_else(|| panic!("Unknown struct type '{}'", type_name));
                    let gep = self.builder
                        .build_struct_gep(st, ptr, idx as u32, "fset")
                        .unwrap();
                    let v = self.gen_expr(val);
                    if is_float {
                        self.builder.build_store(gep, self.as_float(v)).unwrap();
                    } else {
                        self.builder.build_store(gep, self.as_int(v)).unwrap();
                    }
                } else {
                    panic!("Field assignment on non-struct value");
                }
            }

            Stmt::Expr(e) => { self.gen_expr(e); }

            // print expr;
            Stmt::Print(e) => {
                match e {
                    Expr::Str(s) => self.print_str(s),
                    _ => match self.gen_expr(e) {
                        Val::Int(i)      => self.print_int(i),
                        Val::Float(f)    => self.print_float(f),
                        Val::Struct(_, n) => panic!("Cannot print struct '{}' directly", n),
                    }
                }
            }

            // return expr;
            Stmt::Return(e) => {
                let val = self.gen_expr(e);
                let ret = self.as_int(val);
                self.builder.build_return(Some(&ret)).unwrap();
            }

            // { ... }
            Stmt::Block(stmts) => {
                let saved = self.vars.clone();
                for s in stmts {
                    if self.terminated() { break; }
                    self.gen_stmt(s);
                }
                self.vars.retain(|k, _| saved.contains_key(k));
                for (k, v) in &saved {
                    self.vars.entry(k.clone()).or_insert_with(|| v.clone());
                }
            }

            // if (cond) { then } [else { els }]
            Stmt::If { cond, then, els } => {
                let cond_i1  = self.bool_cond(cond);
                let func     = self.cur_fn();
                let then_bb  = self.ctx.append_basic_block(func, "if.then");
                let else_bb  = self.ctx.append_basic_block(func, "if.else");
                let merge_bb = self.ctx.append_basic_block(func, "if.merge");

                self.builder.build_conditional_branch(cond_i1, then_bb, else_bb).unwrap();

                self.builder.position_at_end(then_bb);
                self.gen_stmt(then);
                if !self.terminated() { self.builder.build_unconditional_branch(merge_bb).unwrap(); }

                self.builder.position_at_end(else_bb);
                if let Some(e) = els { self.gen_stmt(e); }
                if !self.terminated() { self.builder.build_unconditional_branch(merge_bb).unwrap(); }

                self.builder.position_at_end(merge_bb);
            }

            // while (cond) { body }
            Stmt::While { cond, body } => {
                let func    = self.cur_fn();
                let hdr_bb  = self.ctx.append_basic_block(func, "while.hdr");
                let body_bb = self.ctx.append_basic_block(func, "while.body");
                let exit_bb = self.ctx.append_basic_block(func, "while.exit");

                self.builder.build_unconditional_branch(hdr_bb).unwrap();

                self.builder.position_at_end(hdr_bb);
                let cond_i1 = self.bool_cond(cond);
                self.builder.build_conditional_branch(cond_i1, body_bb, exit_bb).unwrap();

                self.builder.position_at_end(body_bb);
                self.loop_stack.push((exit_bb, hdr_bb));
                self.gen_stmt(body);
                self.loop_stack.pop();
                if !self.terminated() { self.builder.build_unconditional_branch(hdr_bb).unwrap(); }

                self.builder.position_at_end(exit_bb);
            }

            // for var = from to to { body }
            Stmt::For { var, from, to, body } => {
                let start = { let v = self.gen_expr(from); self.as_int(v) };
                let end   = { let v = self.gen_expr(to);   self.as_int(v) };

                let func     = self.cur_fn();
                let pre_bb   = self.builder.get_insert_block().unwrap();
                let hdr_bb   = self.ctx.append_basic_block(func, "for.hdr");
                let body_bb  = self.ctx.append_basic_block(func, "for.body");
                let step_bb  = self.ctx.append_basic_block(func, "for.step");
                let exit_bb  = self.ctx.append_basic_block(func, "for.exit");

                let loop_alloca = self.builder.build_alloca(self.i64_ty, var).unwrap();

                self.builder.build_unconditional_branch(hdr_bb).unwrap();

                self.builder.position_at_end(hdr_bb);
                let phi   = self.builder.build_phi(self.i64_ty, "for.i").unwrap();
                phi.add_incoming(&[(&start, pre_bb)]);
                let phi_v = phi.as_basic_value().into_int_value();

                self.builder.build_store(loop_alloca, phi_v).unwrap();

                let cond_b = self.builder.build_int_compare(
                    inkwell::IntPredicate::SLE, phi_v, end, "for.cond",
                ).unwrap();
                self.builder.build_conditional_branch(cond_b, body_bb, exit_bb).unwrap();

                self.builder.position_at_end(body_bb);
                let prev = self.vars.insert(
                    var.clone(),
                    Var { ptr: loop_alloca, kind: VarKind::Int },
                );
                self.loop_stack.push((exit_bb, step_bb));
                self.gen_stmt(body);
                self.loop_stack.pop();
                if !self.terminated() { self.builder.build_unconditional_branch(step_bb).unwrap(); }
                match prev {
                    Some(v) => { self.vars.insert(var.clone(), v); }
                    None    => { self.vars.remove(var); }
                }

                self.builder.position_at_end(step_bb);
                let inc = self.builder
                    .build_int_add(phi_v, self.i64_ty.const_int(1, false), "for.inc")
                    .unwrap();
                phi.add_incoming(&[(&inc, step_bb)]);
                self.builder.build_unconditional_branch(hdr_bb).unwrap();

                self.builder.position_at_end(exit_bb);
            }

            // loop { body }
            Stmt::Loop { body } => {
                let func    = self.cur_fn();
                let loop_bb = self.ctx.append_basic_block(func, "loop");
                let exit_bb = self.ctx.append_basic_block(func, "loop.exit");

                self.builder.build_unconditional_branch(loop_bb).unwrap();
                self.builder.position_at_end(loop_bb);

                self.loop_stack.push((exit_bb, loop_bb));
                self.gen_stmt(body);
                self.loop_stack.pop();

                if !self.terminated() { self.builder.build_unconditional_branch(loop_bb).unwrap(); }
                self.builder.position_at_end(exit_bb);
            }

            // break;
            Stmt::Break => {
                let (exit_bb, _) = *self.loop_stack.last()
                    .expect("'break' used outside of a loop");
                self.builder.build_unconditional_branch(exit_bb).unwrap();
            }

            // continue;
            Stmt::Continue => {
                let (_, cont_bb) = *self.loop_stack.last()
                    .expect("'continue' used outside of a loop");
                self.builder.build_unconditional_branch(cont_bb).unwrap();
            }

            // match expr { pat => { body }, ... }
            Stmt::Match { expr, arms } => {
                let val   = self.gen_expr(expr);
                let v     = self.as_int(val);
                let func  = self.cur_fn();
                let merge_bb = self.ctx.append_basic_block(func, "match.end");

                // Pre-allocate arm blocks and check blocks.
                let arm_bbs: Vec<BasicBlock> = (0..arms.len())
                    .map(|i| self.ctx.append_basic_block(func, &format!("match.arm.{}", i)))
                    .collect();
                let check_bbs: Vec<BasicBlock> = (1..arms.len())
                    .map(|i| self.ctx.append_basic_block(func, &format!("match.chk.{}", i)))
                    .collect();

                // Entry block → first arm / check.
                let first_arm = arm_bbs[0];
                if let Some(arm) = arms.first() {
                    match &arm.pat {
                        MatchPat::Int(n) => {
                            let pat_v = self.i64_ty.const_int(*n as u64, true);
                            let cmp   = self.builder.build_int_compare(
                                inkwell::IntPredicate::EQ, v, pat_v, "mc",
                            ).unwrap();
                            let next = if check_bbs.is_empty() { merge_bb } else { check_bbs[0] };
                            self.builder.build_conditional_branch(cmp, first_arm, next).unwrap();
                        }
                        MatchPat::Wildcard => {
                            self.builder.build_unconditional_branch(first_arm).unwrap();
                        }
                    }
                }

                // Generate arm bodies and intermediate checks.
                for (i, arm) in arms.iter().enumerate() {
                    // Emit body of arm i.
                    self.builder.position_at_end(arm_bbs[i]);
                    for s in &arm.body {
                        if self.terminated() { break; }
                        self.gen_stmt(s);
                    }
                    if !self.terminated() { self.builder.build_unconditional_branch(merge_bb).unwrap(); }

                    // Emit check for arm i+1 (if exists).
                    if i + 1 < arms.len() {
                        self.builder.position_at_end(check_bbs[i]);
                        let next_arm = arm_bbs[i + 1];
                        let fall = if i + 1 < check_bbs.len() { check_bbs[i + 1] } else { merge_bb };
                        match &arms[i + 1].pat {
                            MatchPat::Int(n) => {
                                let pat_v = self.i64_ty.const_int(*n as u64, true);
                                let cmp   = self.builder.build_int_compare(
                                    inkwell::IntPredicate::EQ, v, pat_v, "mc",
                                ).unwrap();
                                self.builder.build_conditional_branch(cmp, next_arm, fall).unwrap();
                            }
                            MatchPat::Wildcard => {
                                self.builder.build_unconditional_branch(next_arm).unwrap();
                            }
                        }
                    }
                }

                self.builder.position_at_end(merge_bb);
            }

            Stmt::FnDecl { .. } => {
                panic!("Nested function declarations are not supported");
            }
            Stmt::StructDecl { .. } | Stmt::ImplDecl { .. } => {
                panic!("struct/impl must appear at the top level");
            }
        }
    }

    fn bool_cond(&mut self, cond: &Expr) -> inkwell::values::IntValue<'ctx> {
        let v = self.gen_expr(cond);
        let i = self.as_int(v);
        self.builder.build_int_compare(
            inkwell::IntPredicate::NE, i, self.i64_ty.const_zero(), "cond",
        ).unwrap()
    }

    // ── Expressions ──────────────────────────────────────────────────────

    fn gen_expr(&mut self, expr: &Expr) -> Val<'ctx> {
        match expr {
            Expr::Number(n)  => Val::Int(self.i64_ty.const_int(*n as u64, true)),
            Expr::Float(f)   => Val::Float(self.f64_ty.const_float(*f)),
            Expr::Str(_)     => panic!("String literals can only be used directly in 'print'"),

            Expr::Ident(name) => {
                let var = self.vars.get(name)
                    .cloned()
                    .unwrap_or_else(|| panic!("Undefined variable '{}'", name));
                match var.kind {
                    VarKind::Float =>
                        Val::Float(
                            self.builder.build_load(self.f64_ty, var.ptr, name)
                                .unwrap().into_float_value(),
                        ),
                    VarKind::Int =>
                        Val::Int(
                            self.builder.build_load(self.i64_ty, var.ptr, name)
                                .unwrap().into_int_value(),
                        ),
                    VarKind::Struct(type_name) =>
                        // ptr is already the struct data pointer — no load needed.
                        Val::Struct(var.ptr, type_name),
                }
            }

            // obj.field
            Expr::FieldAccess { obj, field } => {
                let obj_val = self.gen_expr(obj);
                if let Val::Struct(ptr, ref type_name) = obj_val {
                    let type_name = type_name.clone();
                    let field_info = self.struct_fields.get(&type_name)
                        .cloned()
                        .unwrap_or_else(|| panic!("Unknown struct '{}'", type_name));
                    let idx = field_info.iter().position(|(n, _)| n == field)
                        .unwrap_or_else(|| panic!("Unknown field '{}' on '{}'", field, type_name));
                    let (_, is_float) = field_info[idx];
                    let st  = *self.struct_types.get(&type_name)
                        .unwrap_or_else(|| panic!("Unknown struct type '{}'", type_name));
                    let gep = self.builder
                        .build_struct_gep(st, ptr, idx as u32, "fget")
                        .unwrap();
                    if is_float {
                        Val::Float(
                            self.builder.build_load(self.f64_ty, gep, field)
                                .unwrap().into_float_value(),
                        )
                    } else {
                        Val::Int(
                            self.builder.build_load(self.i64_ty, gep, field)
                                .unwrap().into_int_value(),
                        )
                    }
                } else {
                    panic!("Field access on non-struct value");
                }
            }

            // obj.method(args)
            Expr::MethodCall { obj, method, args } => {
                let obj_val = self.gen_expr(obj);
                if let Val::Struct(ptr, ref type_name) = obj_val {
                    let func_name = format!("{}_{}", type_name, method);
                    let callee = self.module.get_function(&func_name)
                        .unwrap_or_else(|| panic!("Unknown method '{}'", func_name));
                    let mut argv: Vec<BasicMetadataValueEnum> = vec![ptr.into()];
                    argv.extend(args.iter().map(|a| {
                        let v = self.gen_expr(a);
                        BasicMetadataValueEnum::IntValue(self.as_int(v))
                    }));
                    let result = self.builder
                        .build_call(callee, &argv, "mcall")
                        .unwrap()
                        .try_as_basic_value()
                        .expect_basic("method must return a value")
                        .into_int_value();
                    Val::Int(result)
                } else {
                    panic!("Method call on non-struct value");
                }
            }

            // new StructName { field: expr, ... }
            Expr::StructLit { name, fields } => {
                let field_info = self.struct_fields.get(name)
                    .cloned()
                    .unwrap_or_else(|| panic!("Unknown struct '{}'", name));
                let st = *self.struct_types.get(name)
                    .unwrap_or_else(|| panic!("Unknown struct type '{}'", name));
                let alloca = self.builder
                    .build_alloca(st, &format!("{}.new", name))
                    .unwrap();
                for (fname, fexpr) in fields {
                    let idx = field_info.iter().position(|(n, _)| n == fname)
                        .unwrap_or_else(|| panic!("Unknown field '{}' on '{}'", fname, name));
                    let (_, is_float) = field_info[idx];
                    let gep = self.builder
                        .build_struct_gep(st, alloca, idx as u32, "finit")
                        .unwrap();
                    let v = self.gen_expr(fexpr);
                    if is_float {
                        self.builder.build_store(gep, self.as_float(v)).unwrap();
                    } else {
                        self.builder.build_store(gep, self.as_int(v)).unwrap();
                    }
                }
                Val::Struct(alloca, name.clone())
            }

            Expr::Unary(op, inner) => {
                let v = self.gen_expr(inner);
                match op {
                    UnaryOp::Neg => match v {
                        Val::Int(i)   => Val::Int(self.builder.build_int_neg(i, "neg").unwrap()),
                        Val::Float(f) => Val::Float(self.builder.build_float_neg(f, "fneg").unwrap()),
                        Val::Struct(_, n) => panic!("Cannot negate struct '{}'", n),
                    },
                    UnaryOp::Not => {
                        let i   = self.as_int(v);
                        let cmp = self.builder.build_int_compare(
                            inkwell::IntPredicate::EQ, i, self.i64_ty.const_zero(), "not.cmp",
                        ).unwrap();
                        Val::Int(self.builder.build_int_z_extend(cmp, self.i64_ty, "not.ext").unwrap())
                    }
                }
            }

            Expr::Binary(lhs, op, rhs) => {
                let l = self.gen_expr(lhs);
                let r = self.gen_expr(rhs);
                self.gen_binop(l, r, op)
            }

            Expr::Call { name, args } => {
                let callee = self.module.get_function(name)
                    .unwrap_or_else(|| panic!("Undefined function '{}'", name));
                let argv: Vec<BasicMetadataValueEnum> = args.iter()
                    .map(|a| {
                        let v = self.gen_expr(a);
                        BasicMetadataValueEnum::IntValue(self.as_int(v))
                    })
                    .collect();
                let result = self.builder
                    .build_call(callee, &argv, "call")
                    .unwrap()
                    .try_as_basic_value()
                    .expect_basic("function must return a value")
                    .into_int_value();
                Val::Int(result)
            }
        }
    }

    fn gen_binop(&mut self, l: Val<'ctx>, r: Val<'ctx>, op: &BinOp) -> Val<'ctx> {
        if l.is_float() || r.is_float() {
            return self.float_binop(self.as_float(l), self.as_float(r), op);
        }
        let (li, ri) = (self.as_int(l), self.as_int(r));
        match op {
            BinOp::Add => Val::Int(self.builder.build_int_add(li, ri, "add").unwrap()),
            BinOp::Sub => Val::Int(self.builder.build_int_sub(li, ri, "sub").unwrap()),
            BinOp::Mul => Val::Int(self.builder.build_int_mul(li, ri, "mul").unwrap()),
            BinOp::Div => Val::Int(self.builder.build_int_signed_div(li, ri, "div").unwrap()),
            BinOp::Mod => Val::Int(self.builder.build_int_signed_rem(li, ri, "mod").unwrap()),
            BinOp::And => Val::Int(self.builder.build_and(li, ri, "and").unwrap()),
            BinOp::Or  => Val::Int(self.builder.build_or(li, ri, "or").unwrap()),
            cmp        => Val::Int(self.int_cmp(li, ri, cmp)),
        }
    }

    fn float_binop(&mut self, l: FloatValue<'ctx>, r: FloatValue<'ctx>, op: &BinOp) -> Val<'ctx> {
        match op {
            BinOp::Add => Val::Float(self.builder.build_float_add(l, r, "fadd").unwrap()),
            BinOp::Sub => Val::Float(self.builder.build_float_sub(l, r, "fsub").unwrap()),
            BinOp::Mul => Val::Float(self.builder.build_float_mul(l, r, "fmul").unwrap()),
            BinOp::Div => Val::Float(self.builder.build_float_div(l, r, "fdiv").unwrap()),
            BinOp::Mod => Val::Float(self.builder.build_float_rem(l, r, "fmod").unwrap()),
            BinOp::And => {
                let (li, ri) = (self.as_int(Val::Float(l)), self.as_int(Val::Float(r)));
                Val::Int(self.builder.build_and(li, ri, "and").unwrap())
            }
            BinOp::Or => {
                let (li, ri) = (self.as_int(Val::Float(l)), self.as_int(Val::Float(r)));
                Val::Int(self.builder.build_or(li, ri, "or").unwrap())
            }
            cmp => Val::Int(self.float_cmp(l, r, cmp)),
        }
    }

    fn int_cmp(&self, l: IntValue<'ctx>, r: IntValue<'ctx>, op: &BinOp) -> IntValue<'ctx> {
        let pred = match op {
            BinOp::Eq => inkwell::IntPredicate::EQ,
            BinOp::Ne => inkwell::IntPredicate::NE,
            BinOp::Lt => inkwell::IntPredicate::SLT,
            BinOp::Le => inkwell::IntPredicate::SLE,
            BinOp::Gt => inkwell::IntPredicate::SGT,
            BinOp::Ge => inkwell::IntPredicate::SGE,
            _         => unreachable!(),
        };
        let bit = self.builder.build_int_compare(pred, l, r, "icmp").unwrap();
        self.builder.build_int_s_extend(bit, self.i64_ty, "icmp.ext").unwrap()
    }

    fn float_cmp(&self, l: FloatValue<'ctx>, r: FloatValue<'ctx>, op: &BinOp) -> IntValue<'ctx> {
        let pred = match op {
            BinOp::Eq => inkwell::FloatPredicate::OEQ,
            BinOp::Ne => inkwell::FloatPredicate::ONE,
            BinOp::Lt => inkwell::FloatPredicate::OLT,
            BinOp::Le => inkwell::FloatPredicate::OLE,
            BinOp::Gt => inkwell::FloatPredicate::OGT,
            BinOp::Ge => inkwell::FloatPredicate::OGE,
            _         => unreachable!(),
        };
        let bit = self.builder.build_float_compare(pred, l, r, "fcmp").unwrap();
        self.builder.build_int_s_extend(bit, self.i64_ty, "fcmp.ext").unwrap()
    }

    // ── printf helpers ────────────────────────────────────────────────────

    fn print_int(&mut self, v: IntValue<'ctx>) {
        let fmt = self.fmt_ptr("%lld\n", "fmt.int");
        let pf  = self.module.get_function("printf").unwrap();
        self.builder.build_call(pf, &[fmt.into(), v.into()], "pr.int").unwrap();
    }

    fn print_float(&mut self, v: FloatValue<'ctx>) {
        let fmt = self.fmt_ptr("%g\n", "fmt.flt");
        let pf  = self.module.get_function("printf").unwrap();
        self.builder.build_call(pf, &[fmt.into(), v.into()], "pr.flt").unwrap();
    }

    fn print_str(&mut self, s: &str) {
        let key  = format!("str.{}", fxhash(s));
        let body = format!("{}\n", s);
        let pf   = self.module.get_function("printf").unwrap();
        let ptr: PointerValue = match self.module.get_global(&key) {
            Some(g) => g.as_pointer_value(),
            None    => self.builder.build_global_string_ptr(&body, &key).unwrap().as_pointer_value(),
        };
        self.builder.build_call(pf, &[ptr.into()], "pr.str").unwrap();
    }

    fn fmt_ptr(&mut self, fmt: &str, name: &str) -> PointerValue<'ctx> {
        match self.module.get_global(name) {
            Some(g) => g.as_pointer_value(),
            None    => self.builder.build_global_string_ptr(fmt, name).unwrap().as_pointer_value(),
        }
    }

    // ── Output: LLVM IR → native binary ──────────────────────────────────

    pub fn save_and_compile(&self, output: &str) -> Result<(), String> {
        self.module
            .print_to_file(Path::new("output.ll"))
            .map_err(|e: LLVMString| format!("IR write failed: {}", e.to_string_lossy()))?;

        let llc = Command::new("llc")
            .args(["output.ll", "-o", "output.s", "-relocation-model=pic"])
            .status()
            .map_err(|e| format!("llc not found: {}", e))?;
        if !llc.success() {
            return Err("llc exited with error".into());
        }

        let cc = Command::new("clang")
            .args(["output.s", "-o", output, "-lm"])
            .status()
            .map_err(|e| format!("clang not found: {}", e))?;
        if !cc.success() {
            return Err("clang exited with error".into());
        }

        println!("Compiled: {}", output);
        Ok(())
    }
}

// Simple hash to give unique names to string globals.
fn fxhash(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}
