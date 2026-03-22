mod expr;
mod stmt;

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
    FloatValue, FunctionValue,
    IntValue, PointerValue,
};

use crate::parser::ast::{Expr, FieldType, MethodDecl, Param, Stmt, UnaryOp};

// ── Compile-time constant value ───────────────────────────────────────────────

#[derive(Clone, Debug)]
pub(super) enum ConstVal {
    Int(i64),
    Float(f64),
}

// ── Variable kind ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub(super) enum VarKind {
    Int,
    Float,
    Struct(String),
    Array,   // flat i64[] array                   (Python / JS)
    FnPtr,   // function pointer (lambda)          (Rust / Python)
    Tuple,   // tuple stored as i64[] (ptr)        (Python / Rust)
}

// ── Variable descriptor ──────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub(super) struct Var<'ctx> {
    pub ptr:  PointerValue<'ctx>,
    pub kind: VarKind,
}

// ── Typed runtime value ──────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub(super) enum Val<'ctx> {
    Int(IntValue<'ctx>),
    Float(FloatValue<'ctx>),
    Struct(PointerValue<'ctx>, String),
    Array(PointerValue<'ctx>),
}

impl<'ctx> Val<'ctx> {
    pub fn is_float(&self) -> bool { matches!(self, Val::Float(_)) }
}

// ── Code generator ───────────────────────────────────────────────────────────

pub struct CodeGen<'ctx> {
    pub(super) ctx:     &'ctx Context,
    pub(super) builder: Builder<'ctx>,
    pub(super) module:  Module<'ctx>,
    pub(super) vars:    HashMap<String, Var<'ctx>>,
    pub(super) i64_ty:  inkwell::types::IntType<'ctx>,
    pub(super) f64_ty:  inkwell::types::FloatType<'ctx>,
    pub(super) struct_types:  HashMap<String, inkwell::types::StructType<'ctx>>,
    pub(super) struct_fields: HashMap<String, Vec<(String, bool)>>,
    pub(super) loop_stack: Vec<(BasicBlock<'ctx>, BasicBlock<'ctx>)>,
    pub(super) enums: HashMap<String, HashMap<String, i64>>,
    pub(super) consts: HashMap<String, ConstVal>,
    pub(super) deferred: Vec<crate::parser::ast::Stmt>,
    /// Tracks array lengths: var_name → length (for ForIn iteration)
    pub(super) array_lens: HashMap<String, i64>,
    /// Counter for unique lambda/generated function names
    pub(super) lambda_counter: usize,
    /// Default param values: fn_name → [Option<Expr>] per param
    pub(super) fn_defaults: HashMap<String, Vec<Option<Expr>>>,
    /// Op-overload registry: type_name → set of method names
    pub(super) op_methods: HashMap<String, Vec<String>>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(module_name: &str, ctx: &'ctx Context) -> Self {
        let module  = ctx.create_module(module_name);
        let builder = ctx.create_builder();
        let i64_ty  = ctx.i64_type();
        let f64_ty  = ctx.f64_type();

        let i8_ptr    = ctx.ptr_type(inkwell::AddressSpace::default());

        // printf / scanf
        let printf_ty = ctx.i32_type().fn_type(&[i8_ptr.into()], true);
        module.add_function("printf", printf_ty, None);
        let scanf_ty = ctx.i32_type().fn_type(&[i8_ptr.into()], true);
        module.add_function("scanf", scanf_ty, None);

        // pow (libm)
        let pow_ty = f64_ty.fn_type(&[f64_ty.into(), f64_ty.into()], false);
        module.add_function("pow", pow_ty, None);

        // syscall(long, ...)
        let syscall_ty = i64_ty.fn_type(&[i64_ty.into()], true);
        module.add_function("syscall", syscall_ty, None);

        // abort() — used by assert                (C standard library)
        let abort_ty = ctx.void_type().fn_type(&[], false);
        module.add_function("abort", abort_ty, None);

        Self {
            ctx,
            builder,
            module,
            vars:           HashMap::new(),
            i64_ty,
            f64_ty,
            struct_types:   HashMap::new(),
            struct_fields:  HashMap::new(),
            loop_stack:     Vec::new(),
            enums:          HashMap::new(),
            consts:         HashMap::new(),
            deferred:       Vec::new(),
            array_lens:     HashMap::new(),
            lambda_counter: 0,
            fn_defaults:    HashMap::new(),
            op_methods:     HashMap::new(),
        }
    }

    // ── Program ──────────────────────────────────────────────────────────────

    pub fn generate_program(&mut self, program: &[Stmt]) {
        // Pass 0: structs, enums, constants, operator-overload registrations
        for stmt in program {
            match stmt {
                Stmt::StructDecl { name, fields } => {
                    self.declare_struct(name, fields);
                }
                Stmt::ClassDecl { name, parent, fields, .. } => {
                    let mut all_fields: Vec<(String, FieldType)> = Vec::new();
                    // Inherit parent fields
                    if let Some(pname) = parent {
                        if let Some(pfields) = self.struct_fields.get(pname).cloned() {
                            for (fname, is_float) in &pfields {
                                all_fields.push((fname.clone(),
                                    if *is_float { FieldType::Float } else { FieldType::Int }));
                            }
                        }
                    }
                    for f in fields {
                        all_fields.push((f.name.clone(), f.ty.clone()));
                    }
                    self.declare_struct(name, &all_fields);
                }
                Stmt::EnumDecl { name, variants } => {
                    let mut map = HashMap::new();
                    for (i, v) in variants.iter().enumerate() {
                        map.insert(v.clone(), i as i64);
                    }
                    self.enums.insert(name.clone(), map);
                }
                Stmt::Const { name, expr } => {
                    self.register_const(name, expr);
                }
                // Register operator-overload methods        (Rust / Swift)
                Stmt::ImplTrait { for_type, methods, .. } => {
                    let names: Vec<String> = methods.iter().map(|m| m.name.clone()).collect();
                    self.op_methods.insert(for_type.clone(), names);
                }
                // Register default param values
                Stmt::FnDecl { name, params, .. } => {
                    let defaults: Vec<Option<Expr>> =
                        params.iter().map(|(_, d)| d.clone()).collect();
                    self.fn_defaults.insert(name.clone(), defaults);
                }
                _ => {}
            }
        }

        // Pass 1: forward-declare all functions / methods
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
                Stmt::ClassDecl { name, methods, .. } => {
                    for m in methods {
                        self.forward_declare_method(name, m);
                    }
                }
                Stmt::ImplTrait { for_type, methods, .. } => {
                    for m in methods {
                        self.forward_declare_method(for_type, m);
                    }
                }
                Stmt::ExternFn { name, params, variadic } => {
                    if self.module.get_function(name).is_none() {
                        let ptys: Vec<BasicMetadataTypeEnum> =
                            (0..*params).map(|_| BasicMetadataTypeEnum::from(self.i64_ty)).collect();
                        self.module.add_function(
                            name,
                            self.i64_ty.fn_type(&ptys, *variadic),
                            None,
                        );
                    }
                }
                _ => {}
            }
        }

        // Pass 2: generate bodies
        for stmt in program {
            match stmt {
                Stmt::FnDecl { name, params, body } => {
                    self.gen_fn(name, params, body);
                }
                Stmt::ImplDecl { struct_name, methods } => {
                    for m in methods { self.gen_method(struct_name, m); }
                }
                Stmt::ClassDecl { name, methods, .. } => {
                    for m in methods { self.gen_method(name, m); }
                }
                Stmt::ImplTrait { for_type, methods, .. } => {
                    for m in methods { self.gen_method(for_type, m); }
                }
                Stmt::StructDecl { .. }
                | Stmt::EnumDecl  { .. }
                | Stmt::Const     { .. }
                | Stmt::Import    { .. }
                | Stmt::ExternFn  { .. }
                | Stmt::TraitDecl { .. }
                | Stmt::Annotation { .. } => {}
                s => panic!("Unexpected top-level statement: {:?}", s),
            }
        }
    }

    // ── Constant registration ─────────────────────────────────────────────────

    fn register_const(&mut self, name: &str, expr: &Expr) {
        match expr {
            Expr::Number(n) => { self.consts.insert(name.to_string(), ConstVal::Int(*n)); }
            Expr::Float(f)  => { self.consts.insert(name.to_string(), ConstVal::Float(*f)); }
            Expr::Unary(UnaryOp::Neg, inner) => match inner.as_ref() {
                Expr::Number(n) => { self.consts.insert(name.to_string(), ConstVal::Int(-n)); }
                Expr::Float(f)  => { self.consts.insert(name.to_string(), ConstVal::Float(-f)); }
                _ => panic!("const '{}' must be a literal value", name),
            },
            _ => panic!("const '{}' must be a literal value (integer or float)", name),
        }
    }

    // ── Struct helpers ────────────────────────────────────────────────────────

    pub(super) fn declare_struct(&mut self, name: &str, fields: &[(String, FieldType)]) {
        let field_types: Vec<inkwell::types::BasicTypeEnum> = fields
            .iter()
            .map(|(_, ft)| match ft {
                FieldType::Int      => self.i64_ty.into(),
                FieldType::Float    => self.f64_ty.into(),
                FieldType::Named(_) => self.i64_ty.into(),
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
        if method.has_self && !method.is_static {
            ptys.push(ptr_ty.into());
        }
        ptys.extend(method.params.iter().map(|_| BasicMetadataTypeEnum::from(self.i64_ty)));
        self.module.add_function(
            &func_name,
            self.i64_ty.fn_type(&ptys, false),
            None,
        );
    }

    // ── Function / Method generation ─────────────────────────────────────────

    pub(super) fn gen_fn(&mut self, name: &str, params: &[Param], body: &[Stmt]) {
        let func = self.module.get_function(name)
            .unwrap_or_else(|| panic!("BUG: function '{}' not forward-declared", name));

        let entry = self.ctx.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);

        let outer_vars     = std::mem::take(&mut self.vars);
        let outer_deferred = std::mem::take(&mut self.deferred);
        let outer_arrays   = std::mem::take(&mut self.array_lens);

        for (i, (pname, _default)) in params.iter().enumerate() {
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
            self.emit_deferred();
            self.builder.build_return(Some(&self.i64_ty.const_int(0, false))).unwrap();
        }

        self.vars      = outer_vars;
        self.deferred  = outer_deferred;
        self.array_lens = outer_arrays;
    }

    pub(super) fn gen_method(&mut self, struct_name: &str, method: &MethodDecl) {
        let func_name = format!("{}_{}", struct_name, method.name);
        let func = self.module.get_function(&func_name)
            .unwrap_or_else(|| panic!("BUG: method '{}' not forward-declared", func_name));

        let entry = self.ctx.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);

        let outer_vars     = std::mem::take(&mut self.vars);
        let outer_deferred = std::mem::take(&mut self.deferred);
        let outer_arrays   = std::mem::take(&mut self.array_lens);
        let mut param_idx  = 0u32;

        if method.has_self && !method.is_static {
            let self_ptr = func.get_nth_param(0).unwrap().into_pointer_value();
            self.vars.insert(
                "self".into(),
                Var { ptr: self_ptr, kind: VarKind::Struct(struct_name.to_string()) },
            );
            param_idx = 1;
        }

        for (i, (pname, _default)) in method.params.iter().enumerate() {
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
            self.emit_deferred();
            self.builder.build_return(Some(&self.i64_ty.const_int(0, false))).unwrap();
        }

        self.vars       = outer_vars;
        self.deferred   = outer_deferred;
        self.array_lens = outer_arrays;
    }

    // ── Defer helpers  (Go) ──────────────────────────────────────────────────

    pub(super) fn emit_deferred(&mut self) {
        let stmts: Vec<Stmt> = self.deferred.iter().rev().cloned().collect();
        for s in &stmts { self.gen_stmt(s); }
    }

    // ── Shared helpers ────────────────────────────────────────────────────────

    pub(super) fn terminated(&self) -> bool {
        self.builder.get_insert_block().and_then(|b| b.get_terminator()).is_some()
    }

    pub(super) fn cur_fn(&self) -> FunctionValue<'ctx> {
        self.builder.get_insert_block().unwrap().get_parent().unwrap()
    }

    pub(super) fn as_int(&self, v: Val<'ctx>) -> IntValue<'ctx> {
        match v {
            Val::Int(i)       => i,
            Val::Float(f)     => self.builder.build_float_to_signed_int(f, self.i64_ty, "f2i").unwrap(),
            Val::Struct(_, n) => panic!("Cannot cast struct '{}' to int", n),
            Val::Array(_)     => panic!("Cannot cast array to int"),
        }
    }

    pub(super) fn as_float(&self, v: Val<'ctx>) -> FloatValue<'ctx> {
        match v {
            Val::Float(f)     => f,
            Val::Int(i)       => self.builder.build_signed_int_to_float(i, self.f64_ty, "i2f").unwrap(),
            Val::Struct(_, n) => panic!("Cannot cast struct '{}' to float", n),
            Val::Array(_)     => panic!("Cannot cast array to float"),
        }
    }

    pub(super) fn bool_cond(&mut self, cond: &Expr) -> IntValue<'ctx> {
        let v = self.gen_expr(cond);
        let i = self.as_int(v);
        self.builder.build_int_compare(inkwell::IntPredicate::NE, i, self.i64_ty.const_zero(), "cond").unwrap()
    }

    // ── printf helpers ────────────────────────────────────────────────────────

    pub(super) fn print_int(&mut self, v: IntValue<'ctx>) {
        let fmt = self.fmt_ptr("%lld\n", "fmt.int");
        let pf  = self.module.get_function("printf").unwrap();
        self.builder.build_call(pf, &[fmt.into(), v.into()], "pr.int").unwrap();
    }

    pub(super) fn print_float(&mut self, v: FloatValue<'ctx>) {
        let fmt = self.fmt_ptr("%g\n", "fmt.flt");
        let pf  = self.module.get_function("printf").unwrap();
        self.builder.build_call(pf, &[fmt.into(), v.into()], "pr.flt").unwrap();
    }

    pub(super) fn print_str(&mut self, s: &str) {
        let key  = format!("str.{}", fxhash(s));
        let body = format!("{}\n", s);
        let pf   = self.module.get_function("printf").unwrap();
        let ptr: PointerValue = match self.module.get_global(&key) {
            Some(g) => g.as_pointer_value(),
            None    => self.builder.build_global_string_ptr(&body, &key).unwrap().as_pointer_value(),
        };
        self.builder.build_call(pf, &[ptr.into()], "pr.str").unwrap();
    }

    pub(super) fn print_interpolated(&mut self, parts: &[crate::lexer::token::InterpolPart]) {
        use crate::lexer::token::InterpolPart;
        use inkwell::values::BasicMetadataValueEnum;

        let mut fmt_str = String::new();
        let mut args: Vec<BasicMetadataValueEnum> = Vec::new();

        for part in parts {
            match part {
                InterpolPart::Lit(s) => {
                    fmt_str.push_str(&s.replace('%', "%%"));
                }
                InterpolPart::Var(name) => {
                    if let Some(var) = self.vars.get(name).cloned() {
                        match var.kind.clone() {
                            VarKind::Float => {
                                fmt_str.push_str("%g");
                                let fv = self.builder.build_load(self.f64_ty, var.ptr, name).unwrap().into_float_value();
                                args.push(fv.into());
                            }
                            VarKind::Int | VarKind::FnPtr => {
                                fmt_str.push_str("%lld");
                                let iv = self.builder.build_load(self.i64_ty, var.ptr, name).unwrap().into_int_value();
                                args.push(iv.into());
                            }
                            VarKind::Array | VarKind::Tuple => {
                                panic!("Arrays/tuples cannot be used in string interpolation directly");
                            }
                            VarKind::Struct(n) => {
                                panic!("Structs ('{}') cannot be used in string interpolation", n);
                            }
                        }
                    } else if let Some(cv) = self.consts.get(name).cloned() {
                        match cv {
                            ConstVal::Int(n) => {
                                fmt_str.push_str("%lld");
                                args.push(self.i64_ty.const_int(n as u64, true).into());
                            }
                            ConstVal::Float(f) => {
                                fmt_str.push_str("%g");
                                args.push(self.f64_ty.const_float(f).into());
                            }
                        }
                    } else {
                        panic!("Undefined variable '{}' in string interpolation", name);
                    }
                }
            }
        }
        fmt_str.push('\n');

        let fmt_name = format!("ifmt.{}", fxhash(&fmt_str));
        let fmt_ptr  = match self.module.get_global(&fmt_name) {
            Some(g) => g.as_pointer_value(),
            None    => self.builder.build_global_string_ptr(&fmt_str, &fmt_name).unwrap().as_pointer_value(),
        };

        let pf = self.module.get_function("printf").unwrap();
        let mut call_args: Vec<BasicMetadataValueEnum> = vec![fmt_ptr.into()];
        call_args.extend(args);
        self.builder.build_call(pf, &call_args, "iprint").unwrap();
    }

    pub(super) fn fmt_ptr(&mut self, fmt: &str, name: &str) -> PointerValue<'ctx> {
        match self.module.get_global(name) {
            Some(g) => g.as_pointer_value(),
            None    => self.builder.build_global_string_ptr(fmt, name).unwrap().as_pointer_value(),
        }
    }

    // ── Output: LLVM IR → native binary ──────────────────────────────────────

    pub fn save_and_compile(&self, output: &str, opts: &CompileOptions) -> Result<(), String> {
        let ll = format!("{}.ll", output);
        let s  = format!("{}.s",  output);

        if opts.verbose { eprintln!("  → Writing LLVM IR: {}", ll); }
        self.module
            .print_to_file(Path::new(&ll))
            .map_err(|e: LLVMString| format!("Failed to write IR: {}", e.to_string_lossy()))?;

        if opts.emit_llvm {
            println!("IR written: {}", ll);
            return Ok(());
        }

        if opts.verbose { eprintln!("  → llc {} → {}", ll, s); }
        let llc_ok = Command::new("llc")
            .args([&ll, "-o", &s, "-relocation-model=pic"])
            .status()
            .map_err(|e| format!("llc not found: {}", e))?;
        if !llc_ok.success() { return Err("llc failed".into()); }

        if opts.verbose { eprintln!("  → clang {} → {}", s, output); }
        let cc_ok = Command::new("clang")
            .args([&s, "-o", output, "-lm"])
            .status()
            .map_err(|e| format!("clang not found: {}", e))?;
        if !cc_ok.success() { return Err("clang failed".into()); }

        if !opts.save_temps {
            let _ = std::fs::remove_file(&ll);
            let _ = std::fs::remove_file(&s);
        }

        println!("Compiled: {}", output);
        Ok(())
    }
}

// ── Compile options ───────────────────────────────────────────────────────────

pub struct CompileOptions {
    pub emit_llvm:  bool,
    pub save_temps: bool,
    pub verbose:    bool,
}

// FNV-1a hash for unique global string names.
pub(super) fn fxhash(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}
