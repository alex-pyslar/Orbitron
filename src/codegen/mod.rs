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

use crate::parser::ast::{Expr, FieldType, MethodDecl, Stmt, UnaryOp};

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
    Array,  // flat i64[] array (from Python / JS)
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
    /// Struct value is always represented as a pointer to its stack allocation.
    Struct(PointerValue<'ctx>, String),
    /// Array value is a pointer to the first i64 element.
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
    /// LLVM struct types keyed by struct name.
    pub(super) struct_types:  HashMap<String, inkwell::types::StructType<'ctx>>,
    /// Ordered field info (name, is_float) keyed by struct name.
    pub(super) struct_fields: HashMap<String, Vec<(String, bool)>>,
    /// Stack of (exit_bb, continue_bb) for the active loop nesting.
    pub(super) loop_stack: Vec<(BasicBlock<'ctx>, BasicBlock<'ctx>)>,
    /// Enum variants: enum_name → { variant_name → integer_value }  (from Rust / Swift)
    pub(super) enums: HashMap<String, HashMap<String, i64>>,
    /// Compile-time constants (from Rust / C++)
    pub(super) consts: HashMap<String, ConstVal>,
    /// Deferred statements to execute at function exit (from Go)
    pub(super) deferred: Vec<crate::parser::ast::Stmt>,
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

        // Declare libm pow(f64, f64) -> f64  — used by ** operator (from Python)
        let pow_ty = f64_ty.fn_type(&[f64_ty.into(), f64_ty.into()], false);
        module.add_function("pow", pow_ty, None);

        // Declare libc syscall(long number, ...) -> long  — for raw syscalls
        let syscall_ty = i64_ty.fn_type(&[i64_ty.into()], true);
        module.add_function("syscall", syscall_ty, None);

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
            enums:         HashMap::new(),
            consts:        HashMap::new(),
            deferred:      Vec::new(),
        }
    }

    // ── Program ──────────────────────────────────────────────────────────────

    pub fn generate_program(&mut self, program: &[Stmt]) {
        // Pass 0: collect struct/class type declarations, enum variants, constants.
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
                // NEW: register enum integer variants  (from Rust / Swift)
                Stmt::EnumDecl { name, variants } => {
                    let mut map = HashMap::new();
                    for (i, v) in variants.iter().enumerate() {
                        map.insert(v.clone(), i as i64);
                    }
                    self.enums.insert(name.clone(), map);
                }
                // NEW: register compile-time constants  (from Rust / C++)
                Stmt::Const { name, expr } => {
                    self.register_const(name, expr);
                }
                _ => {}
            }
        }

        // Pass 1: forward-declare functions, methods and extern declarations.
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
                // extern func — declare external C function (pass 1)
                Stmt::ExternFn { name, params, variadic } => {
                    // Skip if already declared (e.g. printf, scanf, pow, syscall)
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
                // Top-level declarations already handled in pass 0 — skip silently.
                // Import nodes are resolved before codegen — skip silently.
                // ExternFn declared in pass 1 — skip in pass 2.
                Stmt::StructDecl { .. }
                | Stmt::EnumDecl  { .. }
                | Stmt::Const     { .. }
                | Stmt::Import    { .. }
                | Stmt::ExternFn  { .. } => {}
                s => panic!("Неожиданный оператор верхнего уровня: {:?}", s),
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
                _ => panic!("const '{}' должна быть литеральным значением", name),
            },
            _ => panic!("const '{}' должна быть литеральным значением (число или число с плавающей точкой)", name),
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
            .unwrap_or_else(|| panic!("BUG: функция '{}' не была объявлена заранее", name));

        let entry = self.ctx.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);

        let outer_vars     = std::mem::take(&mut self.vars);
        let outer_deferred = std::mem::take(&mut self.deferred); // NEW: defer (Go)

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

        // Emit deferred at implicit function end  (from Go)
        if !self.terminated() {
            self.emit_deferred();
            self.builder
                .build_return(Some(&self.i64_ty.const_int(0, false)))
                .unwrap();
        }

        self.vars     = outer_vars;
        self.deferred = outer_deferred;
    }

    fn gen_method(&mut self, struct_name: &str, method: &MethodDecl) {
        let func_name = format!("{}_{}", struct_name, method.name);
        let func = self.module.get_function(&func_name)
            .unwrap_or_else(|| panic!("BUG: метод '{}' не был объявлен заранее", func_name));

        let entry = self.ctx.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);

        let outer_vars     = std::mem::take(&mut self.vars);
        let outer_deferred = std::mem::take(&mut self.deferred); // NEW: defer (Go)
        let mut param_idx  = 0u32;

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

        // Emit deferred at implicit method end  (from Go)
        if !self.terminated() {
            self.emit_deferred();
            self.builder
                .build_return(Some(&self.i64_ty.const_int(0, false)))
                .unwrap();
        }

        self.vars     = outer_vars;
        self.deferred = outer_deferred;
    }

    // ── Defer helpers  (from Go) ──────────────────────────────────────────────

    /// Emit all deferred statements in LIFO order (last-defer-first).
    pub(super) fn emit_deferred(&mut self) {
        let stmts: Vec<Stmt> = self.deferred.iter().rev().cloned().collect();
        for s in &stmts {
            self.gen_stmt(s);
        }
    }

    // ── Shared helpers ────────────────────────────────────────────────────────

    pub(super) fn terminated(&self) -> bool {
        self.builder
            .get_insert_block()
            .and_then(|b| b.get_terminator())
            .is_some()
    }

    pub(super) fn cur_fn(&self) -> FunctionValue<'ctx> {
        self.builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap()
    }

    pub(super) fn as_int(&self, v: Val<'ctx>) -> IntValue<'ctx> {
        match v {
            Val::Int(i)      => i,
            Val::Float(f)    => self.builder
                .build_float_to_signed_int(f, self.i64_ty, "f2i")
                .unwrap(),
            Val::Struct(_, n) => panic!("Нельзя привести struct '{}' к int", n),
            Val::Array(_)     => panic!("Нельзя привести массив к int"),
        }
    }

    pub(super) fn as_float(&self, v: Val<'ctx>) -> FloatValue<'ctx> {
        match v {
            Val::Float(f)    => f,
            Val::Int(i)      => self.builder
                .build_signed_int_to_float(i, self.f64_ty, "i2f")
                .unwrap(),
            Val::Struct(_, n) => panic!("Нельзя привести struct '{}' к float", n),
            Val::Array(_)     => panic!("Нельзя привести массив к float"),
        }
    }

    pub(super) fn bool_cond(&mut self, cond: &Expr) -> IntValue<'ctx> {
        let v = self.gen_expr(cond);
        let i = self.as_int(v);
        self.builder.build_int_compare(
            inkwell::IntPredicate::NE, i, self.i64_ty.const_zero(), "cond",
        ).unwrap()
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

    /// Print an interpolated string `$"Hello, {name}!"` via printf.
    /// Builds format string at codegen time based on variable types.
    /// (from C# / Kotlin)
    pub(super) fn print_interpolated(&mut self, parts: &[crate::lexer::token::InterpolPart]) {
        use crate::lexer::token::InterpolPart;
        use inkwell::values::BasicMetadataValueEnum;

        let mut fmt_str = String::new();
        let mut args: Vec<BasicMetadataValueEnum> = Vec::new();

        for part in parts {
            match part {
                InterpolPart::Lit(s) => {
                    // Escape '%' so it isn't treated as a format specifier
                    fmt_str.push_str(&s.replace('%', "%%"));
                }
                InterpolPart::Var(name) => {
                    if let Some(var) = self.vars.get(name).cloned() {
                        match var.kind.clone() {
                            VarKind::Float => {
                                fmt_str.push_str("%g");
                                let fv = self.builder
                                    .build_load(self.f64_ty, var.ptr, name)
                                    .unwrap().into_float_value();
                                args.push(fv.into());
                            }
                            VarKind::Int => {
                                fmt_str.push_str("%lld");
                                let iv = self.builder
                                    .build_load(self.i64_ty, var.ptr, name)
                                    .unwrap().into_int_value();
                                args.push(iv.into());
                            }
                            VarKind::Array => {
                                panic!("Массивы не могут быть интерполированы в строке");
                            }
                            VarKind::Struct(n) => {
                                panic!("Структуры ('{}') не могут быть интерполированы в строке", n);
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
                        panic!("Неопределённая переменная '{}' в интерполяции строки", name);
                    }
                }
            }
        }
        fmt_str.push('\n');

        let fmt_name = format!("ifmt.{}", fxhash(&fmt_str));
        let fmt_ptr  = match self.module.get_global(&fmt_name) {
            Some(g) => g.as_pointer_value(),
            None    => self.builder
                .build_global_string_ptr(&fmt_str, &fmt_name)
                .unwrap()
                .as_pointer_value(),
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
pub(super) fn fxhash(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}
