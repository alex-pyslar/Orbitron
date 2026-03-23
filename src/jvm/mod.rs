//! JVM/GraalVM backend — transpiles AST → Java source → javac → .jar
//!
//! Run compiled output:   java -jar <output>.jar
//! GraalVM native image:  native-image -jar <output>.jar -o <output>
//!
//! orbitron.toml:
//!   [build]
//!   backend = "jvm"

use crate::parser::ast::*;
use crate::lexer::token::InterpolPart;
use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::path::Path;
use std::process::Command;

// ── Public options ────────────────────────────────────────────────────────────

pub struct JvmOptions {
    /// --emit-java: write Main.java and stop, don't call javac
    pub emit_java: bool,
    pub verbose:   bool,
}

// ── Public entry point ────────────────────────────────────────────────────────

/// Generate Java source, compile with javac, package as <output>.jar.
pub fn generate_and_compile(
    program: &[Stmt],
    output:  &str,
    opts:    &JvmOptions,
) -> Result<(), String> {
    if opts.verbose { eprintln!("[jvm] Генерация Java источника..."); }

    let java_src = generate_java(program)?;

    let out_path = Path::new(output);
    let out_dir  = out_path.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(out_dir)
        .map_err(|e| format!("Не удалось создать директорию вывода: {e}"))?;

    let java_file = out_dir.join("Main.java");
    std::fs::write(&java_file, &java_src)
        .map_err(|e| format!("Не удалось записать Main.java: {e}"))?;

    if opts.emit_java {
        println!("Java источник записан: {}", java_file.display());
        return Ok(());
    }

    // javac → .class
    if opts.verbose { eprintln!("  → javac {}", java_file.display()); }
    let ok = Command::new("javac")
        .args(["-d", out_dir.to_str().unwrap(), java_file.to_str().unwrap()])
        .status()
        .map_err(|e| format!("javac не найден: {e}"))?;
    if !ok.success() {
        return Err("javac завершился с ошибкой".into());
    }

    // manifest + jar
    let manifest_path = out_dir.join("MANIFEST.MF");
    std::fs::write(&manifest_path, "Main-Class: Main\n")
        .map_err(|e| format!("Не удалось создать MANIFEST.MF: {e}"))?;

    let jar_path = format!("{}.jar", output);
    if opts.verbose { eprintln!("  → jar → {}", jar_path); }
    let ok = Command::new("jar")
        .args([
            "cfm", &jar_path,
            manifest_path.to_str().unwrap(),
            "-C", out_dir.to_str().unwrap(), ".",
        ])
        .status()
        .map_err(|e| format!("jar не найден (нужна JDK): {e}"))?;
    if !ok.success() {
        return Err("jar завершился с ошибкой".into());
    }

    // Cleanup temp files
    let _ = std::fs::remove_file(&manifest_path);
    let _ = std::fs::remove_file(&java_file);
    if let Ok(entries) = std::fs::read_dir(out_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().map(|e| e == "class").unwrap_or(false) {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }

    println!("Скомпилировано (JVM): {}", jar_path);
    Ok(())
}

// ── Code generator ────────────────────────────────────────────────────────────

struct JvmCodeGen {
    /// struct name → ordered (field_name, field_type)
    structs: HashMap<String, Vec<(String, FieldType)>>,
    /// impl/class methods: type_name → methods
    impls:   HashMap<String, Vec<MethodDecl>>,
    /// enum name → variants in declaration order
    enums:   HashMap<String, Vec<String>>,
    /// true while generating main() body (to emit `return;` not `return expr;`)
    in_main: bool,
    /// counter for unique match temp variable names
    match_counter: usize,
}

fn generate_java(program: &[Stmt]) -> Result<String, String> {
    let mut gen = JvmCodeGen {
        structs:       HashMap::new(),
        impls:         HashMap::new(),
        enums:         HashMap::new(),
        in_main:       false,
        match_counter: 0,
    };
    gen.collect(program);
    Ok(gen.emit(program))
}

impl JvmCodeGen {
    // ── Pass 0: collect declarations ─────────────────────────────────────────

    fn collect(&mut self, program: &[Stmt]) {
        for s in program {
            match s {
                Stmt::StructDecl { name, fields } => {
                    self.structs.insert(name.clone(), fields.clone());
                }
                Stmt::ClassDecl { name, methods, .. } => {
                    self.impls.insert(name.clone(), methods.clone());
                }
                Stmt::ImplDecl { struct_name, methods } => {
                    self.impls
                        .entry(struct_name.clone())
                        .or_default()
                        .extend(methods.clone());
                }
                Stmt::EnumDecl { name, variants } => {
                    self.enums.insert(name.clone(), variants.clone());
                }
                _ => {}
            }
        }
    }

    // ── Top-level emitter ─────────────────────────────────────────────────────

    fn emit(&mut self, program: &[Stmt]) -> String {
        let mut out = String::new();

        writeln!(out, "import java.util.Scanner;").unwrap();
        writeln!(out).unwrap();
        writeln!(out, "public class Main {{").unwrap();
        writeln!(out, "    static final Scanner __sc = new Scanner(System.in);").unwrap();
        writeln!(out).unwrap();

        // Enum variants as static final constants
        for (ename, variants) in &self.enums.clone() {
            for (i, v) in variants.iter().enumerate() {
                writeln!(out, "    static final long {}_{} = {}L;", ename, v, i).unwrap();
            }
        }

        // Top-level constants (both old `const` and new `#const` → Stmt::Const)
        for s in program {
            match s {
                Stmt::Const { name, expr } => {
                    let val = self.const_literal(expr);
                    let ty  = if val.contains('.') { "double" } else { "long" };
                    writeln!(out, "    static final {} {} = {};", ty, name, val).unwrap();
                }
                // type aliases → no Java output needed
                Stmt::TypeAlias { .. } => {}
                _ => {}
            }
        }

        // Structs as static inner classes (with all-fields constructor + impl methods)
        for s in program {
            if let Stmt::StructDecl { name, fields } = s {
                self.emit_struct_class(&mut out, name, fields, program);
            }
        }

        // Classes (class + init) as static inner classes
        for s in program {
            if let Stmt::ClassDecl { name, fields, methods, .. } = s {
                self.emit_class_class(&mut out, name, fields, methods);
            }
        }

        // Top-level functions as static methods
        for s in program {
            if let Stmt::FnDecl { name, params, body, .. } = s {
                let param_names: Vec<String> = params.iter().map(|(n, _)| n.clone()).collect();
                self.emit_fn(&mut out, name, &param_names, body);
            }
        }

        writeln!(out, "}}").unwrap();
        out
    }

    // ── Struct → static inner class ───────────────────────────────────────────

    fn emit_struct_class(
        &mut self,
        out:     &mut String,
        name:    &str,
        fields:  &[(String, FieldType)],
        _program: &[Stmt],
    ) {
        writeln!(out, "    static class {} {{", name).unwrap();

        // Fields
        for (fname, ftype) in fields {
            writeln!(out, "        {} {};", ftype_java(ftype), fname).unwrap();
        }

        // All-fields constructor
        let ctor_params: String = fields.iter()
            .map(|(n, t)| format!("{} {}", ftype_java(t), n))
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(out, "        {}({}) {{", name, ctor_params).unwrap();
        for (fname, _) in fields {
            writeln!(out, "            this.{0} = {0};", fname).unwrap();
        }
        writeln!(out, "        }}").unwrap();

        // Methods from impl block
        if let Some(methods) = self.impls.get(name).cloned() {
            for m in &methods {
                self.emit_method(out, &m.clone(), 2);
            }
        }

        writeln!(out, "    }}").unwrap();
        writeln!(out).unwrap();
    }

    // ── Class → static inner class ────────────────────────────────────────────

    fn emit_class_class(
        &mut self,
        out:     &mut String,
        name:    &str,
        fields:  &[FieldDecl],
        methods: &[MethodDecl],
    ) {
        writeln!(out, "    static class {} {{", name).unwrap();

        // Fields
        for f in fields {
            writeln!(out, "        {} {};", ftype_java(&f.ty), f.name).unwrap();
        }

        // Methods (init becomes constructor, others become instance/static methods)
        for m in methods {
            if m.name == "new" {
                // init(...) → constructor
                let params: String = m.params.iter()
                    .map(|(p, _)| format!("long {}", p))
                    .collect::<Vec<_>>()
                    .join(", ");
                writeln!(out, "        {}({}) {{", name, params).unwrap();
                let body_src = self.emit_fn_body(&m.body, 3);
                out.push_str(&body_src);
                writeln!(out, "        }}").unwrap();
            } else {
                self.emit_method(out, &m.clone(), 2);
            }
        }

        writeln!(out, "    }}").unwrap();
        writeln!(out).unwrap();
    }

    // ── Method emitter ────────────────────────────────────────────────────────

    fn emit_method(&mut self, out: &mut String, m: &MethodDecl, depth: usize) {
        let indent  = "    ".repeat(depth);
        let static_ = if m.has_self { "" } else { "static " };
        let params: String = m.params.iter()
            .map(|(p, _)| format!("long {}", p))
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(out, "{}{}long {}({}) {{", indent, static_, m.name, params).unwrap();
        let body_src = self.emit_fn_body(&m.body, depth + 1);
        out.push_str(&body_src);
        // Implicit return 0 (like LLVM backend) — only if last stmt isn't an explicit return
        if !ends_with_return(&m.body) {
            writeln!(out, "{}    return 0L;", indent).unwrap();
        }
        writeln!(out, "{}}}", indent).unwrap();
    }

    // ── Top-level function emitter ────────────────────────────────────────────

    fn emit_fn(&mut self, out: &mut String, name: &str, params: &[String], body: &[Stmt]) {
        if name == "main" {
            writeln!(out, "    public static void main(String[] args) {{").unwrap();
            self.in_main = true;
            let body_src = self.emit_fn_body(body, 2);
            self.in_main = false;
            out.push_str(&body_src);
            writeln!(out, "    }}").unwrap();
        } else {
            let ps: String = params.iter()
                .map(|p| format!("long {}", p))
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(out, "    static long {}({}) {{", name, ps).unwrap();
            let body_src = self.emit_fn_body(body, 2);
            out.push_str(&body_src);
            // Implicit return 0 (like LLVM backend)
            if !ends_with_return(body) {
                writeln!(out, "        return 0L;").unwrap();
            }
            writeln!(out, "    }}").unwrap();
        }
        writeln!(out).unwrap();
    }

    // ── Function body: handle defers via try-finally ──────────────────────────

    fn emit_fn_body(&mut self, body: &[Stmt], depth: usize) -> String {
        // Collect deferred statements (in declaration order; emitted LIFO)
        let mut defers: Vec<Stmt> = Vec::new();
        for s in body {
            if let Stmt::Defer(inner) = s {
                defers.push(*inner.clone());
            }
        }

        if defers.is_empty() {
            return self.emit_stmts(body, depth);
        }

        let indent = "    ".repeat(depth);
        let mut out = String::new();
        writeln!(out, "{}try {{", indent).unwrap();
        out.push_str(&self.emit_stmts(body, depth + 1));
        writeln!(out, "{}}} finally {{", indent).unwrap();
        // LIFO order
        for s in defers.iter().rev() {
            out.push_str(&self.emit_stmt(s, depth + 1));
        }
        writeln!(out, "{}}}", indent).unwrap();
        out
    }

    // ── Statement list ────────────────────────────────────────────────────────

    fn emit_stmts(&mut self, stmts: &[Stmt], depth: usize) -> String {
        let mut out = String::new();
        for s in stmts {
            out.push_str(&self.emit_stmt(s, depth));
        }
        out
    }

    // ── Single statement ──────────────────────────────────────────────────────

    fn emit_stmt(&mut self, stmt: &Stmt, depth: usize) -> String {
        let indent = "    ".repeat(depth);
        let mut out = String::new();

        match stmt {
            Stmt::Let { name, expr } => {
                let ty  = self.infer_java_type(expr);
                let val = self.emit_expr(expr);
                writeln!(out, "{}{} {} = {};", indent, ty, name, val).unwrap();
            }
            // let / mut (new syntax) — same as var in JVM backend
            Stmt::LetNew { name, mutable, expr, .. } => {
                let ty  = self.infer_java_type(expr);
                let val = self.emit_expr(expr);
                if *mutable {
                    writeln!(out, "{}{} {} = {};", indent, ty, name, val).unwrap();
                } else {
                    // immutable → final, but only for primitive types
                    let final_kw = if matches!(ty, "long" | "double") { "final " } else { "" };
                    writeln!(out, "{}{}{} {} = {};", indent, final_kw, ty, name, val).unwrap();
                }
            }
            // type alias — no-op in JVM
            Stmt::TypeAlias { .. } => {}
            Stmt::Const { name, expr } => {
                // const inside a function body → local final
                let val = self.const_literal(expr);
                let ty  = if val.contains('.') { "double" } else { "long" };
                writeln!(out, "{}final {} {} = {};", indent, ty, name, val).unwrap();
            }
            Stmt::Assign { name, expr } => {
                let val = self.emit_expr(expr);
                writeln!(out, "{}{} = {};", indent, name, val).unwrap();
            }
            Stmt::FieldAssign { obj, field, val } => {
                let obj_s = self.emit_expr_self(obj);
                let val_s = self.emit_expr(val);
                writeln!(out, "{}{}.{} = {};", indent, obj_s, field, val_s).unwrap();
            }
            Stmt::IndexAssign { arr, idx, val } => {
                let arr_s = self.emit_expr(arr);
                let idx_s = self.emit_expr(idx);
                let val_s = self.emit_expr(val);
                writeln!(out, "{}{}[(int)({})] = {};", indent, arr_s, idx_s, val_s).unwrap();
            }
            Stmt::Expr(e) => {
                // println!(x) in expression position → print statement
                if let Expr::MacroCall { name, args } = e {
                    if name == "println" {
                        let ea = args.first().cloned().unwrap_or(Expr::Number(0));
                        let p = self.emit_printable(&ea);
                        writeln!(out, "{}System.out.println({});", indent, p).unwrap();
                        return out;
                    }
                }
                let e_s = self.emit_expr(e);
                writeln!(out, "{}{};", indent, e_s).unwrap();
            }
            Stmt::Print(e) => {
                let p = self.emit_printable(e);
                writeln!(out, "{}System.out.println({});", indent, p).unwrap();
            }
            Stmt::Return(e) => {
                if self.in_main {
                    // void main → discard return value
                    let val = self.emit_expr(e);
                    // Emit as expression statement only if it has side effects
                    // (simple numbers are safe to discard silently)
                    match e {
                        Expr::Number(_) | Expr::Float(_) => {}
                        _ => writeln!(out, "{}{};", indent, val).unwrap(),
                    }
                    writeln!(out, "{}return;", indent).unwrap();
                } else {
                    let val = self.emit_expr(e);
                    writeln!(out, "{}return {};", indent, val).unwrap();
                }
            }
            Stmt::Block(stmts) => {
                writeln!(out, "{}{{", indent).unwrap();
                out.push_str(&self.emit_stmts(stmts, depth + 1));
                writeln!(out, "{}}}", indent).unwrap();
            }
            Stmt::If { cond, then, els } => {
                let c = self.emit_cond(cond);
                writeln!(out, "{}if ({}) {{", indent, c).unwrap();
                out.push_str(&self.emit_body(then, depth));
                writeln!(out, "{}}}", indent).unwrap();
                if let Some(e) = els {
                    writeln!(out, "{}else {{", indent).unwrap();
                    out.push_str(&self.emit_body(e, depth));
                    writeln!(out, "{}}}", indent).unwrap();
                }
            }
            Stmt::While { cond, body } => {
                let c = self.emit_cond(cond);
                writeln!(out, "{}while ({}) {{", indent, c).unwrap();
                out.push_str(&self.emit_body(body, depth));
                writeln!(out, "{}}}", indent).unwrap();
            }
            Stmt::DoWhile { body, cond } => {
                writeln!(out, "{}do {{", indent).unwrap();
                out.push_str(&self.emit_body(body, depth));
                let c = self.emit_cond(cond);
                writeln!(out, "{}}} while ({});", indent, c).unwrap();
            }
            Stmt::For { var, from, to, inclusive, body } => {
                let from_s = self.emit_expr(from);
                let to_s   = self.emit_expr(to);
                let cmp    = if *inclusive { "<=" } else { "<" };
                writeln!(
                    out,
                    "{}for (long {} = {}; {} {} {}; {}++) {{",
                    indent, var, from_s, var, cmp, to_s, var
                ).unwrap();
                out.push_str(&self.emit_body(body, depth));
                writeln!(out, "{}}}", indent).unwrap();
            }
            Stmt::Loop { body } => {
                writeln!(out, "{}while (true) {{", indent).unwrap();
                out.push_str(&self.emit_body(body, depth));
                writeln!(out, "{}}}", indent).unwrap();
            }
            Stmt::Break    => { writeln!(out, "{}break;",    indent).unwrap(); }
            Stmt::Continue => { writeln!(out, "{}continue;", indent).unwrap(); }
            Stmt::Match { expr, arms } => {
                // Each match gets a unique variable name + block scope to avoid
                // "duplicate local variable" errors from multiple match stmts.
                let mv = format!("__m{}", self.match_counter);
                self.match_counter += 1;
                let e_s = self.emit_expr(expr);
                writeln!(out, "{}{{", indent).unwrap();
                writeln!(out, "{}    long {} = {};", indent, mv, e_s).unwrap();
                let mut first = true;
                for arm in arms {
                    match &arm.pat {
                        MatchPat::Wildcard => {
                            writeln!(out, "{}    else {{", indent).unwrap();
                            out.push_str(&self.emit_stmts(&arm.body, depth + 2));
                            writeln!(out, "{}    }}", indent).unwrap();
                        }
                        MatchPat::Int(n) => {
                            let kw = if first { "if" } else { "else if" };
                            writeln!(out, "{}    {} ({} == {}L) {{", indent, kw, mv, n).unwrap();
                            out.push_str(&self.emit_stmts(&arm.body, depth + 2));
                            writeln!(out, "{}    }}", indent).unwrap();
                            first = false;
                        }
                        MatchPat::EnumVariant(ename, variant) => {
                            let kw = if first { "if" } else { "else if" };
                            writeln!(out, "{}    {} ({} == {}_{}) {{", indent, kw, mv, ename, variant).unwrap();
                            out.push_str(&self.emit_stmts(&arm.body, depth + 2));
                            writeln!(out, "{}    }}", indent).unwrap();
                            first = false;
                        }
                    }
                }
                writeln!(out, "{}}}", indent).unwrap();
            }
            // var (a, b) = expr;  — tuple destructuring  (Python / Rust)
            Stmt::LetTuple { names, expr } => {
                let e_s = self.emit_expr(expr);
                writeln!(out, "{}long[] __tup_{} = {};", indent, self.match_counter, e_s).unwrap();
                for (i, name) in names.iter().enumerate() {
                    writeln!(out, "{}long {} = __tup_{}[{}];", indent, name, self.match_counter, i).unwrap();
                }
                self.match_counter += 1;
            }
            // for x in array { body }  — array iteration  (Python)
            Stmt::ForIn { var, iter, body } => {
                let iter_s = self.emit_expr(iter);
                writeln!(out, "{}for (long {} : {}) {{", indent, var, iter_s).unwrap();
                out.push_str(&self.emit_body(body, depth));
                writeln!(out, "{}}}", indent).unwrap();
            }
            // @annotation — no-op in JVM backend
            Stmt::Annotation { .. } => {}
            // trait / impl Trait — no-op (methods are emitted via ImplDecl)
            Stmt::TraitDecl { .. } | Stmt::ImplTrait { .. } => {}
            // Already handled at top level or structurally irrelevant here
            Stmt::FnDecl { .. }
            | Stmt::StructDecl { .. }
            | Stmt::ImplDecl { .. }
            | Stmt::ClassDecl { .. }
            | Stmt::EnumDecl { .. }
            | Stmt::Import { .. }
            | Stmt::Defer(_)
            | Stmt::ExternFn { .. } => {}
        }
        out
    }

    /// Emit the interior of a block-or-single-statement body (braces already written by caller).
    fn emit_body(&mut self, stmt: &Stmt, depth: usize) -> String {
        match stmt {
            Stmt::Block(stmts) => self.emit_stmts(stmts, depth + 1),
            _ => self.emit_stmt(stmt, depth + 1),
        }
    }

    // ── Expression emitter ────────────────────────────────────────────────────

    fn emit_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Number(n)       => format!("{}L", n),
            Expr::Float(f)        => format!("{}d", f),
            Expr::Str(s)          => format!("\"{}\"", escape_java_str(s)),
            Expr::Interpolated(p) => self.emit_interp(p),
            Expr::Ident(name)     => self.map_ident(name),
            Expr::Binary(l, op, r) => self.emit_binop(l, op, r),
            Expr::Unary(op, inner) => {
                let s = self.emit_expr(inner);
                match op {
                    UnaryOp::Neg    => format!("(-{})", s),
                    UnaryOp::Not    => format!("(({}) == 0L ? 1L : 0L)", s),
                    UnaryOp::BitNot => format!("(~({}))", s),  // bitwise NOT  (C / Java)
                }
            }
            Expr::Ternary { cond, then, els } => {
                let c = self.emit_cond(cond);
                let t = self.emit_expr(then);
                let e = self.emit_expr(els);
                format!("({} ? {} : {})", c, t, e)
            }
            Expr::Call { name, args } => {
                let args_s = self.emit_args(args);
                format!("{}({})", name, args_s)
            }
            Expr::FieldAccess { obj, field } => {
                // Enum variant access: Season.Summer → Season_Summer
                if let Expr::Ident(name) = obj.as_ref() {
                    if self.enums.contains_key(name.as_str()) {
                        return format!("{}_{}", name, field);
                    }
                }
                let o = self.emit_expr_self(obj);
                format!("{}.{}", o, field)
            }
            Expr::MethodCall { obj, method, args } => {
                let o     = self.emit_expr_self(obj);
                let args_s = self.emit_args(args);
                format!("{}.{}({})", o, method, args_s)
            }
            Expr::StructLit { name, fields } => {
                // Reorder fields to match constructor declaration order
                let ordered = if let Some(decl_fields) = self.structs.get(name).cloned() {
                    decl_fields.iter()
                        .map(|(fname, _)| {
                            fields.iter()
                                .find(|(n, _)| n == fname)
                                .map(|(_, e)| self.emit_expr(e))
                                .unwrap_or_else(|| "0L".to_string())
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                } else {
                    self.emit_field_args(fields)
                };
                format!("new {}({})", name, ordered)
            }
            Expr::ConstructorCall { class, args } => {
                let args_s = self.emit_args(args);
                format!("new {}({})", class, args_s)
            }
            Expr::ArrayLit(elems) => {
                let elems_s = self.emit_args(elems);
                format!("new long[]{{ {} }}", elems_s)
            }
            Expr::Index { arr, idx } => {
                let a = self.emit_expr(arr);
                let i = self.emit_expr(idx);
                format!("{}[(int)({})]", a, i)
            }
            Expr::Input      => "__sc.nextLong()".to_string(),
            Expr::InputFloat => "__sc.nextDouble()".to_string(),
            // Type::method(args)  — static call  (C++ / Rust)
            Expr::StaticCall { type_name, method, args } => {
                let args_s = self.emit_args(args);
                format!("{}.{}({})", type_name, method, args_s)
            }
            // (a, b)  — tuple literal emitted as long[]  (Python / Rust)
            Expr::Tuple(exprs) => {
                let elems_s = self.emit_args(exprs);
                format!("new long[]{{ {} }}", elems_s)
            }
            // |params| expr  — lambda as Java lambda  (Rust / Python)
            Expr::Lambda { params, body } => {
                let ps = params.join(", ");
                let b  = self.emit_expr(body);
                format!("(({}) -> {})", ps, b)
            }
            // match expr { pat => val, ... }  — ternary chain  (Rust)
            Expr::MatchExpr { expr, arms } => {
                let e_s = self.emit_expr(expr);
                let mut result = String::new();
                // Build a nested ternary: (e == pat ? val : (e == pat2 ? val2 : default))
                let mut chain = String::from("0L");
                for arm in arms.iter().rev() {
                    let v_s = self.emit_expr(&arm.val);
                    chain = match &arm.pat {
                        MatchPat::Wildcard                    => v_s,
                        MatchPat::Int(n)                      => format!("(({}) == {}L ? {} : {})", e_s, n, v_s, chain),
                        MatchPat::EnumVariant(en, var)        => format!("(({}) == {}_{} ? {} : {})", e_s, en, var, v_s, chain),
                    };
                }
                result.push_str(&chain);
                result
            }
            // name!(args) — macro call: dispatch same as regular call in JVM
            Expr::MacroCall { name, args } => {
                let args_s = self.emit_args(args);
                format!("{}({})", name, args_s)
            }
            // left ?: right — Elvis / null-coalescing  (Kotlin)
            Expr::Elvis { left, right } => {
                let l = self.emit_expr(left);
                let r = self.emit_expr(right);
                format!("(({}) != 0L ? ({}) : ({}))", l, l, r)
            }
            // expr?.field — optional chaining  (Swift / Kotlin)
            Expr::OptChain { expr: inner, field } => {
                let obj = self.emit_expr(inner);
                format!("(({}) != 0L ? ({}).{} : 0L)", obj, obj, field)
            }
            // Low-level features not supported in the JVM backend
            Expr::AddrOf(_) =>
                panic!("&addr_of не поддерживается в JVM-бекенде (используйте --backend llvm)"),
            Expr::Deref(_) =>
                panic!("*deref не поддерживается в JVM-бекенде (используйте --backend llvm)"),
            Expr::CStr(_) =>
                panic!("cstr() не поддерживается в JVM-бекенде (используйте --backend llvm)"),
        }
    }

    /// Like emit_expr but maps `self` → `this` for object receiver expressions.
    fn emit_expr_self(&mut self, expr: &Expr) -> String {
        let s = self.emit_expr(expr);
        if s == "self" { "this".to_string() } else { s }
    }

    /// Emit a binary operation. Comparisons return `long` (1L/0L) for expression context.
    fn emit_binop(&mut self, l: &Expr, op: &BinOp, r: &Expr) -> String {
        let l_s = self.emit_expr(l);
        let r_s = self.emit_expr(r);
        match op {
            BinOp::Add => format!("({} + {})", l_s, r_s),
            BinOp::Sub => format!("({} - {})", l_s, r_s),
            BinOp::Mul => format!("({} * {})", l_s, r_s),
            BinOp::Div => format!("({} / {})", l_s, r_s),
            BinOp::Mod => format!("({} % {})", l_s, r_s),
            BinOp::Pow => format!("(long)Math.pow((double)({}), (double)({}))", l_s, r_s),
            BinOp::Eq  => format!("(({}) == ({}) ? 1L : 0L)", l_s, r_s),
            BinOp::Ne  => format!("(({}) != ({}) ? 1L : 0L)", l_s, r_s),
            BinOp::Gt  => format!("(({}) >  ({}) ? 1L : 0L)", l_s, r_s),
            BinOp::Lt  => format!("(({}) <  ({}) ? 1L : 0L)", l_s, r_s),
            BinOp::Ge  => format!("(({}) >= ({}) ? 1L : 0L)", l_s, r_s),
            BinOp::Le  => format!("(({}) <= ({}) ? 1L : 0L)", l_s, r_s),
            BinOp::And => format!("(({}) != 0L && ({}) != 0L ? 1L : 0L)", l_s, r_s),
            BinOp::Or  => format!("(({}) != 0L || ({}) != 0L ? 1L : 0L)", l_s, r_s),
            BinOp::Xor => format!("(({}) ^ ({}))", l_s, r_s),  // bitwise XOR  (C / Java)
        }
    }

    /// Emit an expression as a Java boolean condition (for if/while/for).
    fn emit_cond(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Binary(l, op, r) => {
                let l_s = self.emit_expr(l);
                let r_s = self.emit_expr(r);
                match op {
                    BinOp::Eq  => format!("({}) == ({})", l_s, r_s),
                    BinOp::Ne  => format!("({}) != ({})", l_s, r_s),
                    BinOp::Gt  => format!("({}) >  ({})", l_s, r_s),
                    BinOp::Lt  => format!("({}) <  ({})", l_s, r_s),
                    BinOp::Ge  => format!("({}) >= ({})", l_s, r_s),
                    BinOp::Le  => format!("({}) <= ({})", l_s, r_s),
                    BinOp::And => {
                        let lc = self.emit_cond(l);
                        let rc = self.emit_cond(r);
                        format!("({}) && ({})", lc, rc)
                    }
                    BinOp::Or => {
                        let lc = self.emit_cond(l);
                        let rc = self.emit_cond(r);
                        format!("({}) || ({})", lc, rc)
                    }
                    _ => {
                        let s = self.emit_binop(l, op, r);
                        format!("({}) != 0L", s)
                    }
                }
            }
            Expr::Unary(UnaryOp::Not, inner) => {
                let c = self.emit_cond(inner);
                format!("!({})", c)
            }
            _ => {
                let s = self.emit_expr(expr);
                format!("({}) != 0L", s)
            }
        }
    }

    /// Emit `println` argument — interpolated strings become string concatenation.
    fn emit_printable(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Interpolated(parts) => self.emit_interp(parts),
            _ => self.emit_expr(expr),
        }
    }

    /// Emit string interpolation `$"Hello {name}!"` as Java string concatenation.
    fn emit_interp(&mut self, parts: &[InterpolPart]) -> String {
        if parts.is_empty() { return "\"\"".to_string(); }
        let pieces: Vec<String> = parts.iter().map(|p| match p {
            InterpolPart::Lit(s) => format!("\"{}\"", escape_java_str(s)),
            InterpolPart::Var(v) => self.map_ident(v),
        }).collect();
        if pieces.len() == 1 {
            pieces.into_iter().next().unwrap()
        } else {
            format!("({})", pieces.join(" + "))
        }
    }

    fn emit_args(&mut self, args: &[Expr]) -> String {
        args.iter().map(|a| self.emit_expr(a)).collect::<Vec<_>>().join(", ")
    }

    fn emit_field_args(&mut self, fields: &[(String, Expr)]) -> String {
        fields.iter().map(|(_, e)| self.emit_expr(e)).collect::<Vec<_>>().join(", ")
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// `self` → `this`; enum dot-access `Dir.North` → `Dir_North`.
    fn map_ident(&self, name: &str) -> String {
        if name == "self" { return "this".to_string(); }
        // Enum dot-access is parsed as Expr::FieldAccess, not Ident, so no special case needed here.
        name.to_string()
    }

    /// Infer the Java type name for a `var` declaration.
    fn infer_java_type(&self, expr: &Expr) -> &'static str {
        match expr {
            Expr::Float(_) | Expr::InputFloat => "double",
            Expr::ArrayLit(_)                 => "long[]",
            Expr::StructLit { .. } | Expr::ConstructorCall { .. } => "var",
            _ => "long",
        }
    }

    /// Evaluate a const expression to a Java literal string.
    fn const_literal(&self, expr: &Expr) -> String {
        match expr {
            Expr::Number(n) => format!("{}L", n),
            Expr::Float(f)  => format!("{}d", f),
            Expr::Unary(UnaryOp::Neg, inner) => {
                let s = self.const_literal(inner);
                format!("-{}", s)
            }
            _ => panic!("const должна быть литеральным значением"),
        }
    }
}

// ── Utility functions ─────────────────────────────────────────────────────────

/// True if `body`'s last statement is a `return`, meaning the implicit
/// `return 0L;` would cause an "unreachable statement" Java compile error.
fn ends_with_return(body: &[Stmt]) -> bool {
    matches!(body.last(), Some(Stmt::Return(_)))
}

fn ftype_java(ft: &FieldType) -> String {
    match ft {
        FieldType::Int      => "long".to_string(),
        FieldType::Float    => "double".to_string(),
        FieldType::Named(t) => t.clone(),
    }
}

fn escape_java_str(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('"',  "\\\"")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
     .replace('\t', "\\t")
}
