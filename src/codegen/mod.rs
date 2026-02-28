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

use crate::parser::ast::{Expr, FieldType, MethodDecl, Stmt};

// ── Variable kind ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
enum VarKind {
    Int,
    Float,
    Struct(String),
}

// ── Variable descriptor ──────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct Var<'ctx> {
    ptr:  PointerValue<'ctx>,
    kind: VarKind,
}

// ── Typed runtime value ──────────────────────────────────────────────────────

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

// ── Code generator ───────────────────────────────────────────────────────────

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

        // Declare libc scanf: int scanf(char*, ...)
        let scanf_ty = ctx.i32_type().fn_type(&[i8_ptr.into()], true);
        module.add_function("scanf", scanf_ty, None);

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

    // ── Program ──────────────────────────────────────────────────────────────

    pub fn generate_program(&mut self, program: &[Stmt]) {
        // Pass 0: collect struct/class type declarations.
        for stmt in program {
            match stmt {
                Stmt::StructDecl { name, fields } => {
                    self.declare_struct(name, fields);
                }
                Stmt::ClassDecl { name, fields, .. } => {
                    let tuples: Vec<(String, FieldType)> = fields
                        .iter()
                        .map(|f| (f.name.clone(), f.ty.clone()))
                        .collect();
                    self.declare_struct(name, &tuples);
                }
                _ => {}
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
                Stmt::ClassDecl { name, methods, .. } => {
                    for m in methods {
                        self.forward_declare_method(name, m);
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
                Stmt::ClassDecl { name, methods, .. } => {
                    for m in methods {
                        self.gen_method(name, m);
                    }
                }
                Stmt::StructDecl { .. } => {}
                s => panic!("Unexpected top-level statement: {:?}", s),
            }
        }
    }

    // ── Struct helpers ────────────────────────────────────────────────────────

    fn declare_struct(&mut self, name: &str, fields: &[(String, FieldType)]) {
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

    // ── Function / Method generation ─────────────────────────────────────────

    fn gen_fn(&mut self, name: &str, params: &[String], body: &[Stmt]) {
        let func = self.module.get_function(name)
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

    fn gen_method(&mut self, struct_name: &str, method: &MethodDecl) {
        let func_name = format!("{}_{}", struct_name, method.name);
        let func = self.module.get_function(&func_name)
            .unwrap_or_else(|| panic!("BUG: method '{}' not pre-declared", func_name));

        let entry = self.ctx.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);

        let outer_vars = std::mem::take(&mut self.vars);
        let mut param_idx = 0u32;

        if method.has_self {
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

    // ── Shared helpers ────────────────────────────────────────────────────────

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
            Val::Struct(_, n) => panic!("Нельзя привести struct '{}' к int", n),
        }
    }

    fn as_float(&self, v: Val<'ctx>) -> FloatValue<'ctx> {
        match v {
            Val::Float(f) => f,
            Val::Int(i)   => self.builder
                .build_signed_int_to_float(i, self.f64_ty, "i2f")
                .unwrap(),
            Val::Struct(_, n) => panic!("Нельзя привести struct '{}' к float", n),
        }
    }

    fn bool_cond(&mut self, cond: &Expr) -> IntValue<'ctx> {
        let v = self.gen_expr(cond);
        let i = self.as_int(v);
        self.builder.build_int_compare(
            inkwell::IntPredicate::NE, i, self.i64_ty.const_zero(), "cond",
        ).unwrap()
    }

    // ── printf helpers ────────────────────────────────────────────────────────

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

    // ── Output: LLVM IR → native binary ──────────────────────────────────────

    pub fn save_and_compile(&self, output: &str, opts: &CompileOptions) -> Result<(), String> {
        let ll = format!("{}.ll", output);
        let s  = format!("{}.s",  output);

        // Write LLVM IR
        if opts.verbose { eprintln!("  → Запись LLVM IR: {}", ll); }
        self.module
            .print_to_file(Path::new(&ll))
            .map_err(|e: LLVMString| format!("Запись IR не удалась: {}", e.to_string_lossy()))?;

        if opts.emit_llvm {
            println!("IR записан: {}", ll);
            return Ok(());
        }

        // IR → assembly
        if opts.verbose { eprintln!("  → llc {} → {}", ll, s); }
        let llc_ok = Command::new("llc")
            .args([&ll, "-o", &s, "-relocation-model=pic"])
            .status()
            .map_err(|e| format!("llc не найден: {}", e))?;
        if !llc_ok.success() {
            return Err("llc завершился с ошибкой".into());
        }

        // assembly → binary
        if opts.verbose { eprintln!("  → clang {} → {}", s, output); }
        let cc_ok = Command::new("clang")
            .args([&s, "-o", output, "-lm"])
            .status()
            .map_err(|e| format!("clang не найден: {}", e))?;
        if !cc_ok.success() {
            return Err("clang завершился с ошибкой".into());
        }

        // Clean up intermediate files unless --save-temps
        if !opts.save_temps {
            let _ = std::fs::remove_file(&ll);
            let _ = std::fs::remove_file(&s);
        }

        println!("Скомпилировано: {}", output);
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
fn fxhash(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}
