//! orbitron fmt — canonical code formatter (gofmt-style).
//!
//! Reads a .ot source file, parses it, then pretty-prints the AST
//! back to text using a single canonical style:
//!   - 4-space indentation
//!   - spaces around binary operators
//!   - opening `{` on the same line
//!   - one blank line between top-level declarations

use crate::parser::ast::*;
use crate::lexer::token::InterpolPart;

// ── Public entry point ────────────────────────────────────────────────────────

/// Format an Orbitron source file.
/// Returns the formatted source text on success.
pub fn format_source(src: &str) -> Result<String, String> {
    let tokens = crate::lexer::Lexer::tokenize(src)?;
    let program = crate::parser::Parser::new(tokens).parse_program()?;
    Ok(fmt_program(&program))
}

// ── Program ───────────────────────────────────────────────────────────────────

fn fmt_program(stmts: &[Stmt]) -> String {
    let mut out   = String::new();
    let mut first = true;
    for stmt in stmts {
        let s = fmt_stmt(stmt, 0);
        if s.trim().is_empty() { continue; }
        if !first { out.push('\n'); }
        out.push_str(&s);
        first = false;
    }
    out
}

// ── Statements ────────────────────────────────────────────────────────────────

fn fmt_stmt(stmt: &Stmt, depth: usize) -> String {
    let ind = indent(depth);
    match stmt {
        Stmt::Let { name, expr } =>
            format!("{}var {} = {};\n", ind, name, fmt_expr(expr)),

        Stmt::LetTuple { names, expr } =>
            format!("{}var ({}) = {};\n", ind, names.join(", "), fmt_expr(expr)),

        Stmt::Const { name, expr } =>
            format!("{}const {} = {};\n", ind, name, fmt_expr(expr)),

        Stmt::Assign { name, expr } =>
            format!("{}{} = {};\n", ind, name, fmt_expr(expr)),

        Stmt::FieldAssign { obj, field, val } =>
            format!("{}{}.{} = {};\n", ind, fmt_expr(obj), field, fmt_expr(val)),

        Stmt::IndexAssign { arr, idx, val } =>
            format!("{}{}[{}] = {};\n", ind, fmt_expr(arr), fmt_expr(idx), fmt_expr(val)),

        Stmt::Expr(e) =>
            format!("{}{};\n", ind, fmt_expr(e)),

        Stmt::Print(e) =>
            format!("{}println({});\n", ind, fmt_print_arg(e)),

        Stmt::Return(e) =>
            format!("{}return {};\n", ind, fmt_expr(e)),

        Stmt::Block(stmts) => {
            let mut s = format!("{}{{\n", ind);
            for st in stmts { s.push_str(&fmt_stmt(st, depth + 1)); }
            s.push_str(&format!("{}}}\n", ind));
            s
        }

        Stmt::If { cond, then, els } => {
            let mut s = format!("{}if ({}) {{\n", ind, fmt_expr(cond));
            s.push_str(&fmt_block_body(then, depth));
            s.push_str(&format!("{}}}", ind));
            if let Some(e) = els {
                s.push_str(" else {\n");
                s.push_str(&fmt_block_body(e, depth));
                s.push_str(&format!("{}}}", ind));
            }
            s.push('\n');
            s
        }

        Stmt::While { cond, body } => {
            let mut s = format!("{}while ({}) {{\n", ind, fmt_expr(cond));
            s.push_str(&fmt_block_body(body, depth));
            s.push_str(&format!("{}}}\n", ind));
            s
        }

        Stmt::DoWhile { body, cond } => {
            let mut s = format!("{}do {{\n", ind);
            s.push_str(&fmt_block_body(body, depth));
            s.push_str(&format!("{}}} while ({});\n", ind, fmt_expr(cond)));
            s
        }

        Stmt::For { var, from, to, inclusive, body } => {
            let range = if *inclusive {
                format!("{}..={}", fmt_expr(from), fmt_expr(to))
            } else {
                format!("{}..{}", fmt_expr(from), fmt_expr(to))
            };
            let mut s = format!("{}for {} in {} {{\n", ind, var, range);
            s.push_str(&fmt_block_body(body, depth));
            s.push_str(&format!("{}}}\n", ind));
            s
        }

        Stmt::ForIn { var, iter, body } => {
            let mut s = format!("{}for {} in {} {{\n", ind, var, fmt_expr(iter));
            s.push_str(&fmt_block_body(body, depth));
            s.push_str(&format!("{}}}\n", ind));
            s
        }

        Stmt::Loop { body } => {
            let mut s = format!("{}loop {{\n", ind);
            s.push_str(&fmt_block_body(body, depth));
            s.push_str(&format!("{}}}\n", ind));
            s
        }

        Stmt::Break    => format!("{}break;\n",    ind),
        Stmt::Continue => format!("{}continue;\n", ind),

        Stmt::Match { expr, arms } => {
            let mut s = format!("{}match {} {{\n", ind, fmt_expr(expr));
            for arm in arms {
                let pat = fmt_match_pat(&arm.pat);
                s.push_str(&format!("{}    {} => {{\n", ind, pat));
                for st in &arm.body {
                    s.push_str(&fmt_stmt(st, depth + 2));
                }
                s.push_str(&format!("{}    }}\n", ind));
            }
            s.push_str(&format!("{}}}\n", ind));
            s
        }

        Stmt::FnDecl { name, params, body } => {
            let ps = fmt_params(params);
            let mut s = format!("{}func {}({}) {{\n", ind, name, ps);
            for st in body { s.push_str(&fmt_stmt(st, depth + 1)); }
            s.push_str(&format!("{}}}\n", ind));
            s
        }

        Stmt::StructDecl { name, fields } => {
            let mut s = format!("{}struct {} {{\n", ind, name);
            for (fname, ftype) in fields {
                s.push_str(&format!("{}    {}: {},\n", ind, fname, fmt_field_type(ftype)));
            }
            s.push_str(&format!("{}}}\n", ind));
            s
        }

        Stmt::ImplDecl { struct_name, methods } => {
            let mut s = format!("{}impl {} {{\n", ind, struct_name);
            for m in methods { s.push_str(&fmt_method(m, depth + 1)); }
            s.push_str(&format!("{}}}\n", ind));
            s
        }

        Stmt::ImplTrait { trait_name, for_type, methods } => {
            let mut s = format!("{}impl {} for {} {{\n", ind, trait_name, for_type);
            for m in methods { s.push_str(&fmt_method(m, depth + 1)); }
            s.push_str(&format!("{}}}\n", ind));
            s
        }

        Stmt::TraitDecl { name, methods } => {
            let mut s = format!("{}trait {} {{\n", ind, name);
            for (mname, params) in methods {
                let ps = params.join(", ");
                s.push_str(&format!("{}    func {}({});\n", ind, mname, ps));
            }
            s.push_str(&format!("{}}}\n", ind));
            s
        }

        Stmt::ClassDecl { name, parent, fields, methods } => {
            let ext = parent.as_ref()
                .map(|p| format!(" extends {}", p))
                .unwrap_or_default();
            let mut s = format!("{}class{}{} {{\n", ind, ext, name);
            // Reorder: fields first
            for f in fields {
                let acc = match f.access { Access::Public => "pub ", Access::Private => "private " };
                s.push_str(&format!("{}    {}{}: {},\n", ind, acc, f.name, fmt_field_type(&f.ty)));
            }
            for m in methods { s.push_str(&fmt_method(m, depth + 1)); }
            s.push_str(&format!("{}}}\n", ind));
            s
        }

        Stmt::EnumDecl { name, variants } => {
            let vs: Vec<String> = variants.iter().map(|v| format!("{}    {}", ind, v)).collect();
            format!("{}enum {} {{\n{}\n{}}}\n", ind, name, vs.join(",\n"), ind)
        }

        Stmt::Defer(s) =>
            format!("{}defer {};\n", ind, fmt_stmt_inline(s)),

        Stmt::Import { path } =>
            format!("{}import \"{}\";\n", ind, path),

        Stmt::ExternFn { name, params, variadic } => {
            let ellipsis = if *variadic { ", ..." } else { "" };
            let ps: String = (0..*params)
                .map(|i| format!("p{}: int", i))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}extern func {}({}{});\n", ind, name, ps, ellipsis)
        }

        Stmt::Annotation { name } =>
            format!("{}@{}\n", ind, name),
    }
}

// ── Expressions ───────────────────────────────────────────────────────────────

fn fmt_expr(expr: &Expr) -> String {
    match expr {
        Expr::Number(n)      => n.to_string(),
        Expr::Float(f)       => {
            let s = format!("{}", f);
            if s.contains('.') { s } else { format!("{}.0", s) }
        }
        Expr::Str(s)         => format!("\"{}\"", escape_str(s)),
        Expr::Interpolated(parts) => {
            let inner: String = parts.iter().map(|p| match p {
                InterpolPart::Lit(s) => escape_str(s),
                InterpolPart::Var(v) => format!("{{{}}}", v),
            }).collect();
            format!("$\"{}\"", inner)
        }
        Expr::Ident(n)       => n.clone(),
        Expr::Binary(l, op, r) =>
            format!("({} {} {})", fmt_expr(l), fmt_binop(op), fmt_expr(r)),
        Expr::Unary(op, e)   => match op {
            UnaryOp::Neg    => format!("(-{})",  fmt_expr(e)),
            UnaryOp::Not    => format!("(!{})",  fmt_expr(e)),
            UnaryOp::BitNot => format!("(~{})",  fmt_expr(e)),
        },
        Expr::Ternary { cond, then, els } =>
            format!("{} ? {} : {}", fmt_expr(cond), fmt_expr(then), fmt_expr(els)),
        Expr::Call { name, args } => {
            let as_ = args.iter().map(fmt_expr).collect::<Vec<_>>().join(", ");
            format!("{}({})", name, as_)
        }
        Expr::StaticCall { type_name, method, args } => {
            let as_ = args.iter().map(fmt_expr).collect::<Vec<_>>().join(", ");
            format!("{}::{}({})", type_name, method, as_)
        }
        Expr::FieldAccess { obj, field } =>
            format!("{}.{}", fmt_expr(obj), field),
        Expr::MethodCall { obj, method, args } => {
            let as_ = args.iter().map(fmt_expr).collect::<Vec<_>>().join(", ");
            format!("{}.{}({})", fmt_expr(obj), method, as_)
        }
        Expr::StructLit { name, fields } => {
            let fs: Vec<String> = fields.iter()
                .map(|(n, e)| format!("{}: {}", n, fmt_expr(e)))
                .collect();
            format!("{} {{ {} }}", name, fs.join(", "))
        }
        Expr::ConstructorCall { class, args } => {
            let as_ = args.iter().map(fmt_expr).collect::<Vec<_>>().join(", ");
            format!("new {}({})", class, as_)
        }
        Expr::ArrayLit(elems) => {
            let es = elems.iter().map(fmt_expr).collect::<Vec<_>>().join(", ");
            format!("[{}]", es)
        }
        Expr::Index { arr, idx } =>
            format!("{}[{}]", fmt_expr(arr), fmt_expr(idx)),
        Expr::Tuple(elems) => {
            let es = elems.iter().map(fmt_expr).collect::<Vec<_>>().join(", ");
            format!("({})", es)
        }
        Expr::Lambda { params, body } =>
            format!("|{}| {}", params.join(", "), fmt_expr(body)),
        Expr::MatchExpr { expr, arms } => {
            let mut s = format!("match {} {{ ", fmt_expr(expr));
            let arm_strs: Vec<String> = arms.iter()
                .map(|a| format!("{} => {}", fmt_match_pat(&a.pat), fmt_expr(&a.val)))
                .collect();
            s.push_str(&arm_strs.join(", "));
            s.push_str(" }");
            s
        }
        Expr::Input      => "readInt()".to_string(),
        Expr::InputFloat => "readFloat()".to_string(),
        Expr::AddrOf(e)  => format!("&{}", fmt_expr(e)),
        Expr::Deref(e)   => format!("*{}", fmt_expr(e)),
        Expr::CStr(s)    => format!("cstr(\"{}\")", escape_str(s)),
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn indent(depth: usize) -> String {
    "    ".repeat(depth)
}

fn fmt_binop(op: &BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",  BinOp::Sub => "-",  BinOp::Mul => "*",
        BinOp::Div => "/",  BinOp::Mod => "%",  BinOp::Pow => "**",
        BinOp::Xor => "^",
        BinOp::Gt  => ">",  BinOp::Lt  => "<",  BinOp::Ge  => ">=",
        BinOp::Le  => "<=", BinOp::Eq  => "==", BinOp::Ne  => "!=",
        BinOp::And => "&&", BinOp::Or  => "||",
    }
}

fn fmt_field_type(ft: &FieldType) -> &str {
    match ft {
        FieldType::Int      => "int",
        FieldType::Float    => "float",
        FieldType::Named(n) => n.as_str(),
    }
}

fn fmt_params(params: &[Param]) -> String {
    params.iter().map(|(name, default)| {
        if let Some(d) = default {
            format!("{} = {}", name, fmt_expr(d))
        } else {
            name.clone()
        }
    }).collect::<Vec<_>>().join(", ")
}

fn fmt_method(m: &MethodDecl, depth: usize) -> String {
    let ind   = indent(depth);
    let acc   = match m.access { Access::Public => "pub ", Access::Private => "private " };
    let stat  = if m.is_static { "static " } else { "" };
    let self_ = if m.has_self && !m.is_static { "self" } else { "" };
    let ps    = fmt_params(&m.params);
    let full_params = if self_.is_empty() { ps } else if ps.is_empty() { self_.to_string() }
                      else { format!("{}, {}", self_, ps) };
    let mut s = format!("{}{}{}func {}({}) {{\n", ind, acc, stat, m.name, full_params);
    for st in &m.body { s.push_str(&fmt_stmt(st, depth + 1)); }
    s.push_str(&format!("{}}}\n", ind));
    s
}

fn fmt_match_pat(pat: &MatchPat) -> String {
    match pat {
        MatchPat::Int(n)                => n.to_string(),
        MatchPat::Wildcard              => "_".to_string(),
        MatchPat::EnumVariant(e, v)     => format!("{}.{}", e, v),
    }
}

fn fmt_block_body(stmt: &Stmt, depth: usize) -> String {
    match stmt {
        Stmt::Block(stmts) => {
            let mut s = String::new();
            for st in stmts { s.push_str(&fmt_stmt(st, depth + 1)); }
            s
        }
        _ => fmt_stmt(stmt, depth + 1),
    }
}

/// Format a statement as a single-line expression (for `defer`).
fn fmt_stmt_inline(stmt: &Stmt) -> String {
    match stmt {
        Stmt::Expr(e)     => fmt_expr(e),
        Stmt::Print(e)    => format!("println({})", fmt_print_arg(e)),
        Stmt::Return(e)   => format!("return {}", fmt_expr(e)),
        _                 => fmt_stmt(stmt, 0).trim_end_matches('\n').to_string(),
    }
}

fn fmt_print_arg(e: &Expr) -> String {
    match e {
        Expr::Str(s)         => format!("\"{}\"", escape_str(s)),
        Expr::Interpolated(_) => fmt_expr(e),
        _                    => fmt_expr(e),
    }
}

fn escape_str(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('"',  "\\\"")
     .replace('\n', "\\n")
     .replace('\t', "\\t")
}
