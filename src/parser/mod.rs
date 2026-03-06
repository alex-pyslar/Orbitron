pub mod ast;
pub use ast::*;

use crate::lexer::Token;

// ── Parser ──────────────────────────────────────────────────────────────────

pub struct Parser {
    tokens: Vec<Token>,
    pos:    usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    // ── Token stream helpers ───────────────────────────────────────────────

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn peek2(&self) -> &Token {
        self.tokens.get(self.pos + 1).unwrap_or(&Token::Eof)
    }

    fn peek3(&self) -> &Token {
        self.tokens.get(self.pos + 2).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let t = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        if self.pos < self.tokens.len() { self.pos += 1; }
        t
    }

    fn check(&self, tok: &Token) -> bool { self.peek() == tok }

    fn eat(&mut self, tok: &Token) -> bool {
        if self.peek() == tok { self.advance(); true } else { false }
    }

    fn expect(&mut self, tok: &Token) -> Result<(), String> {
        let got = self.advance();
        if &got == tok { Ok(()) }
        else { Err(format!("Ожидалось {:?}, получено {:?}", tok, got)) }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.advance() {
            Token::Ident(s) => Ok(s),
            t => Err(format!("Ожидался идентификатор, получено {:?}", t)),
        }
    }

    /// Like `expect_ident` but also accepts keyword `new` as a method name.
    fn expect_method_name(&mut self) -> Result<String, String> {
        match self.advance() {
            Token::Ident(s) => Ok(s),
            Token::New      => Ok("new".to_string()),
            t => Err(format!("Ожидалось имя метода, получено {:?}", t)),
        }
    }

    // ── Top level ──────────────────────────────────────────────────────────

    pub fn parse_program(&mut self) -> Result<Vec<Stmt>, String> {
        let mut items = Vec::new();
        while !self.check(&Token::Eof) {
            items.push(self.parse_top_level()?);
        }
        Ok(items)
    }

    fn parse_top_level(&mut self) -> Result<Stmt, String> {
        match self.peek() {
            Token::Func   => self.parse_fn_decl(),
            Token::Struct => self.parse_struct_decl(),
            Token::Impl   => self.parse_impl_decl(),
            Token::Class  => self.parse_class_decl(),
            Token::Enum   => self.parse_enum_decl(),    // NEW: Rust/Swift enums
            Token::Const  => self.parse_const(),        // NEW: Rust/C++ constants
            Token::Import => self.parse_import(),       // NEW: multi-file import
            Token::Extern => self.parse_extern_fn(),    // NEW: extern C function
            t => Err(format!(
                "Ожидалось 'func', 'struct', 'impl', 'class', 'enum', 'const', 'import' \
                 или 'extern' на верхнем уровне, получено {:?}",
                t.clone()
            )),
        }
    }

    fn parse_import(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'import'
        let path = match self.advance() {
            Token::Str(s) => s,
            t => return Err(format!("import ожидает строку, получено {:?}", t)),
        };
        self.eat(&Token::Semicolon);
        Ok(Stmt::Import { path })
    }

    /// `extern func name(p0: type, p1: type [, ...]): ret;`
    /// Declares an external C function. All parameter types are treated as i64.
    /// Use `...` as the last parameter for variadic functions (like printf).
    fn parse_extern_fn(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'extern'
        self.expect(&Token::Func)?;
        let name = self.expect_ident()?;
        self.expect(&Token::LParen)?;

        let mut params: usize = 0;
        let mut variadic = false;

        if !self.check(&Token::RParen) {
            loop {
                // `...` variadic marker
                if self.check(&Token::DotDot) {
                    // consume .. then check for next dot for ...
                    self.advance();
                    if self.eat(&Token::Dot) {
                        variadic = true;
                        break;
                    }
                    return Err("Ожидалось '...' (три точки) для variadic".into());
                }
                // named param: ident [: type]
                self.expect_ident()?;
                if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
                params += 1;
                if !self.eat(&Token::Comma) { break; }
            }
        }

        self.expect(&Token::RParen)?;
        // optional return type annotation
        if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
        self.eat(&Token::Semicolon);
        Ok(Stmt::ExternFn { name, params, variadic })
    }

    fn parse_fn_decl(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'func'
        let name = self.expect_ident()?;
        self.expect(&Token::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(&Token::RParen)?;
        // optional return-type annotation : type
        if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
        let body = self.parse_block_stmts()?;
        Ok(Stmt::FnDecl { name, params, body })
    }

    fn parse_param_list(&mut self) -> Result<Vec<String>, String> {
        let mut params = Vec::new();
        if self.check(&Token::RParen) { return Ok(params); }
        params.push(self.expect_ident()?);
        if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
        while self.eat(&Token::Comma) {
            params.push(self.expect_ident()?);
            if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
        }
        Ok(params)
    }

    /// Skip over a type name token (int / float / identifier).
    /// Type annotations are parsed but not enforced at codegen level.
    fn skip_type_annotation(&mut self) -> Result<(), String> {
        match self.peek().clone() {
            Token::Ident(_) => { self.advance(); Ok(()) }
            t => Err(format!("Ожидалось имя типа, получено {:?}", t)),
        }
    }

    // ── Struct / Impl ──────────────────────────────────────────────────────

    fn parse_struct_decl(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'struct'
        let name = self.expect_ident()?;
        self.expect(&Token::LBrace)?;
        let mut fields = Vec::new();
        while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
            let fname = self.expect_ident()?;
            self.expect(&Token::Colon)?;
            let ftype = self.parse_field_type()?;
            fields.push((fname, ftype));
            self.eat(&Token::Comma);
        }
        self.expect(&Token::RBrace)?;
        Ok(Stmt::StructDecl { name, fields })
    }

    fn parse_field_type(&mut self) -> Result<FieldType, String> {
        match self.peek().clone() {
            Token::Ident(s) => {
                self.advance();
                match s.as_str() {
                    "int"   => Ok(FieldType::Int),
                    "float" => Ok(FieldType::Float),
                    other   => Ok(FieldType::Named(other.to_string())),
                }
            }
            t => Err(format!("Ожидался тип поля (int/float/имя), получено {:?}", t)),
        }
    }

    fn parse_impl_decl(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'impl'
        let struct_name = self.expect_ident()?;
        self.expect(&Token::LBrace)?;
        let mut methods = Vec::new();
        while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
            let access = self.parse_access_modifier();
            let mut m = self.parse_method_decl()?;
            m.access = access;
            methods.push(m);
        }
        self.expect(&Token::RBrace)?;
        Ok(Stmt::ImplDecl { struct_name, methods })
    }

    /// `class Name { [pub|private] field: type, ...  init(...) { }  pub func method(...) { ... } }`
    fn parse_class_decl(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'class'
        let name = self.expect_ident()?;
        self.expect(&Token::LBrace)?;

        let mut fields  = Vec::new();
        let mut methods = Vec::new();

        while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
            let access = self.parse_access_modifier();

            if self.check(&Token::Func) || self.check(&Token::Init) {
                // method or constructor declaration
                let mut m = self.parse_method_decl()?;
                m.access = access;
                methods.push(m);
            } else {
                // field declaration:  name: type [,]
                let fname = self.expect_ident()?;
                self.expect(&Token::Colon)?;
                let ftype = self.parse_field_type()?;
                self.eat(&Token::Comma);
                fields.push(FieldDecl { name: fname, ty: ftype, access });
            }
        }

        self.expect(&Token::RBrace)?;
        Ok(Stmt::ClassDecl { name, fields, methods })
    }

    /// `enum Name { Variant, ... }`  (from Rust / Swift)
    /// Each variant maps to an integer (0, 1, 2, …).
    fn parse_enum_decl(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'enum'
        let name = self.expect_ident()?;
        self.expect(&Token::LBrace)?;
        let mut variants = Vec::new();
        while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
            variants.push(self.expect_ident()?);
            self.eat(&Token::Comma);
        }
        self.expect(&Token::RBrace)?;
        Ok(Stmt::EnumDecl { name, variants })
    }

    /// `const NAME [: type] = expr;`  (from Rust / C++)
    fn parse_const(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'const'
        let name = self.expect_ident()?;
        if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
        self.expect(&Token::Assign)?;
        let expr = self.parse_expr()?;
        self.expect(&Token::Semicolon)?;
        Ok(Stmt::Const { name, expr })
    }

    /// Consume an optional `pub` or `private` keyword and return the access level.
    fn parse_access_modifier(&mut self) -> Access {
        if self.eat(&Token::Pub)          { Access::Public  }
        else if self.eat(&Token::Private) { Access::Private }
        else                              { Access::Public  }
    }

    fn parse_method_decl(&mut self) -> Result<MethodDecl, String> {
        // Accept 'func' or 'init' (constructor shorthand without explicit self)
        let is_init = self.check(&Token::Init);
        if is_init {
            self.advance(); // consume 'init'
        } else {
            self.expect(&Token::Func)?;
        }

        // For 'init', the internal method name is "new" (reuses constructor codegen)
        let name = if is_init {
            "new".to_string()
        } else {
            self.expect_method_name()?
        };

        self.expect(&Token::LParen)?;

        let mut has_self = is_init; // init always has implicit self
        let mut params   = Vec::new();

        if is_init {
            // init(params)  — self is implicit, don't expect Token::SelfKw
            if !self.check(&Token::RParen) {
                params.push(self.expect_ident()?);
                if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
                while self.eat(&Token::Comma) {
                    params.push(self.expect_ident()?);
                    if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
                }
            }
        } else {
            // pub func method(self, ...)  or  pub func method(a, b)
            if !self.check(&Token::RParen) {
                if self.check(&Token::SelfKw) {
                    self.advance(); // consume 'self'
                    has_self = true;
                    if self.eat(&Token::Comma) {
                        params.push(self.expect_ident()?);
                        if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
                        while self.eat(&Token::Comma) {
                            params.push(self.expect_ident()?);
                            if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
                        }
                    }
                } else {
                    params.push(self.expect_ident()?);
                    if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
                    while self.eat(&Token::Comma) {
                        params.push(self.expect_ident()?);
                        if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
                    }
                }
            }
        }

        self.expect(&Token::RParen)?;
        if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
        let body = self.parse_block_stmts()?;
        Ok(MethodDecl { name, params, has_self, body, access: Access::Public })
    }

    // ── Block ──────────────────────────────────────────────────────────────

    fn parse_block_stmts(&mut self) -> Result<Vec<Stmt>, String> {
        self.expect(&Token::LBrace)?;
        let mut stmts = Vec::new();
        while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(&Token::RBrace)?;
        Ok(stmts)
    }

    // ── Statements ─────────────────────────────────────────────────────────

    /// Returns true when the token sequence is  (ident | self) DOT ident ASSIGN
    fn is_field_assign_stmt(&self) -> bool {
        let first_ok = matches!(
            self.tokens.get(self.pos),
            Some(Token::Ident(_)) | Some(Token::SelfKw)
        );
        let dot_ok    = matches!(self.tokens.get(self.pos + 1), Some(Token::Dot));
        let field_ok  = matches!(self.tokens.get(self.pos + 2), Some(Token::Ident(_)));
        let assign_ok = matches!(self.tokens.get(self.pos + 3), Some(Token::Assign));
        first_ok && dot_ok && field_ok && assign_ok
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        if self.is_field_assign_stmt() {
            return self.parse_field_assign_stmt();
        }

        match self.peek().clone() {
            Token::Var      => self.parse_var(),
            Token::Const    => self.parse_const(),              // NEW: Rust/C++
            Token::If       => self.parse_if(),
            Token::Unless   => self.parse_unless(),             // NEW: Ruby
            Token::While    => self.parse_while(),
            Token::Do       => self.parse_do_while(),
            Token::For      => self.parse_for(),
            Token::Loop     => self.parse_loop(),
            Token::Repeat   => self.parse_repeat(),             // NEW
            Token::Defer    => self.parse_defer_stmt(),         // NEW: Go
            Token::Return   => self.parse_return(),
            Token::Println  => self.parse_println(),
            Token::Match    => self.parse_match(),
            Token::Break    => {
                self.advance();
                self.expect(&Token::Semicolon)?;
                Ok(Stmt::Break)
            }
            Token::Continue => {
                self.advance();
                self.expect(&Token::Semicolon)?;
                Ok(Stmt::Continue)
            }
            Token::LBrace => {
                let body = self.parse_block_stmts()?;
                Ok(Stmt::Block(body))
            }
            // compound assignment:  ident op= expr ;
            Token::Ident(_) if matches!(
                self.peek2(),
                Token::PlusAssign | Token::MinusAssign |
                Token::StarAssign | Token::SlashAssign
            ) => self.parse_compound_assign(),
            // simple assignment:  ident = expr ;
            Token::Ident(_) if matches!(self.peek2(), Token::Assign) => {
                self.parse_assign()
            }
            // expression statement (may be index assignment: arr[i] = val;)
            _ => {
                let e = self.parse_expr()?;
                // Check for index assignment: arr[i] = val;
                if self.eat(&Token::Assign) {
                    let val = self.parse_expr()?;
                    self.expect(&Token::Semicolon)?;
                    match e {
                        Expr::Index { arr, idx } =>
                            return Ok(Stmt::IndexAssign { arr, idx, val }),
                        _ => return Err(
                            "Недопустимая левая часть присваивания (ожидался arr[idx])".into()
                        ),
                    }
                }
                self.expect(&Token::Semicolon)?;
                Ok(Stmt::Expr(e))
            }
        }
    }

    /// `var name [: type] = expr;`
    fn parse_var(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'var'
        let name = self.expect_ident()?;
        if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
        self.expect(&Token::Assign)?;
        let expr = self.parse_expr()?;
        self.expect(&Token::Semicolon)?;
        Ok(Stmt::Let { name, expr })
    }

    fn parse_assign(&mut self) -> Result<Stmt, String> {
        let name = self.expect_ident()?;
        self.expect(&Token::Assign)?;
        let expr = self.parse_expr()?;
        self.expect(&Token::Semicolon)?;
        Ok(Stmt::Assign { name, expr })
    }

    /// Desugar:  name op= rhs  →  name = name op rhs
    fn parse_compound_assign(&mut self) -> Result<Stmt, String> {
        let name = self.expect_ident()?;
        let op = match self.advance() {
            Token::PlusAssign  => BinOp::Add,
            Token::MinusAssign => BinOp::Sub,
            Token::StarAssign  => BinOp::Mul,
            Token::SlashAssign => BinOp::Div,
            t => return Err(format!("Ожидался оператор присваивания, получено {:?}", t)),
        };
        let rhs = self.parse_expr()?;
        self.expect(&Token::Semicolon)?;
        Ok(Stmt::Assign {
            name: name.clone(),
            expr: Expr::Binary(Box::new(Expr::Ident(name)), op, Box::new(rhs)),
        })
    }

    fn parse_field_assign_stmt(&mut self) -> Result<Stmt, String> {
        let obj_name = match self.advance() {
            Token::Ident(s) => s,
            Token::SelfKw   => "self".to_string(),
            t => return Err(format!("Ожидался идентификатор или 'self', получено {:?}", t)),
        };
        self.expect(&Token::Dot)?;
        let field = self.expect_ident()?;
        self.expect(&Token::Assign)?;
        let val = self.parse_expr()?;
        self.expect(&Token::Semicolon)?;
        Ok(Stmt::FieldAssign { obj: Expr::Ident(obj_name), field, val })
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'if'
        self.expect(&Token::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(&Token::RParen)?;
        let then_body = self.parse_block_stmts()?;
        let then = Box::new(Stmt::Block(then_body));

        let els = if self.eat(&Token::Else) {
            if self.check(&Token::If) {
                Some(Box::new(self.parse_if()?))
            } else {
                let body = self.parse_block_stmts()?;
                Some(Box::new(Stmt::Block(body)))
            }
        } else {
            None
        };

        Ok(Stmt::If { cond, then, els })
    }

    /// `unless (cond) { body }`  (from Ruby)
    /// Desugars to: `if (!cond) { body }`
    fn parse_unless(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'unless'
        self.expect(&Token::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(&Token::RParen)?;
        let body = self.parse_block_stmts()?;
        Ok(Stmt::If {
            cond: Expr::Unary(UnaryOp::Not, Box::new(cond)),
            then: Box::new(Stmt::Block(body)),
            els:  None,
        })
    }

    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'while'
        self.expect(&Token::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(&Token::RParen)?;
        let body = self.parse_block_stmts()?;
        Ok(Stmt::While { cond, body: Box::new(Stmt::Block(body)) })
    }

    /// `do { body } while (cond);`
    fn parse_do_while(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'do'
        let body = self.parse_block_stmts()?;
        self.expect(&Token::While)?;
        self.expect(&Token::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(&Token::RParen)?;
        self.expect(&Token::Semicolon)?;
        Ok(Stmt::DoWhile { body: Box::new(Stmt::Block(body)), cond })
    }

    /// `for i in start..end { }` or `for i in start..=end { }`
    /// Multiple ranges: `for i in 0..3, j in 0..5 { }` → nested loops
    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'for'

        // Collect one or more range specs separated by commas
        let mut ranges = vec![self.parse_range_spec()?];
        while self.eat(&Token::Comma) {
            ranges.push(self.parse_range_spec()?);
        }

        let body_stmts = self.parse_block_stmts()?;

        // Desugar from inside out: innermost range wraps the body
        let innermost = Stmt::Block(body_stmts);
        let result = ranges.into_iter().rev().fold(innermost, |inner, (var, from, to, inclusive)| {
            Stmt::For { var, from, to, inclusive, body: Box::new(inner) }
        });
        Ok(result)
    }

    /// Parse one `ident in expr..expr` or `ident in expr..=expr`
    fn parse_range_spec(&mut self) -> Result<(String, Expr, Expr, bool), String> {
        let var  = self.expect_ident()?;
        self.expect(&Token::In)?;
        let from = self.parse_expr()?;
        let inclusive = if self.eat(&Token::DotDotEq) {
            true
        } else if self.eat(&Token::DotDot) {
            false
        } else {
            return Err(format!(
                "Ожидался '..' или '..=' в цикле for, получено {:?}",
                self.peek().clone()
            ));
        };
        let to = self.parse_expr()?;
        Ok((var, from, to, inclusive))
    }

    fn parse_loop(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'loop'
        let body = self.parse_block_stmts()?;
        Ok(Stmt::Loop { body: Box::new(Stmt::Block(body)) })
    }

    /// `repeat N { body }`  (inspired by Lua / Pascal)
    /// Desugars to: `for __ri in 0..N { body }`
    fn parse_repeat(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'repeat'
        let count = self.parse_expr()?;
        let body  = self.parse_block_stmts()?;
        Ok(Stmt::For {
            var:       "__ri".to_string(),
            from:      Expr::Number(0),
            to:        count,
            inclusive: false,
            body:      Box::new(Stmt::Block(body)),
        })
    }

    /// `defer stmt;`  (from Go) — registers statement for execution at function exit.
    /// Accepts expression-statements and `println(...)` calls.
    fn parse_defer_stmt(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'defer'
        let inner = match self.peek() {
            Token::Println => self.parse_println()?,
            _ => {
                let e = self.parse_expr()?;
                self.expect(&Token::Semicolon)?;
                Stmt::Expr(e)
            }
        };
        Ok(Stmt::Defer(Box::new(inner)))
    }

    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'return'
        if self.eat(&Token::Semicolon) {
            Ok(Stmt::Return(Expr::Number(0)))
        } else {
            let e = self.parse_expr()?;
            self.expect(&Token::Semicolon)?;
            Ok(Stmt::Return(e))
        }
    }

    /// `println(expr);`
    fn parse_println(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'println'
        self.expect(&Token::LParen)?;
        let e = self.parse_expr()?;
        self.expect(&Token::RParen)?;
        self.expect(&Token::Semicolon)?;
        Ok(Stmt::Print(e))
    }

    fn parse_match(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'match'
        let expr = self.parse_expr()?;
        self.expect(&Token::LBrace)?;
        let mut arms = Vec::new();
        while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
            arms.push(self.parse_match_arm()?);
        }
        self.expect(&Token::RBrace)?;
        Ok(Stmt::Match { expr, arms })
    }

    fn parse_match_arm(&mut self) -> Result<MatchArm, String> {
        let pat = match self.peek().clone() {
            Token::Int(n) => {
                self.advance();
                MatchPat::Int(n)
            }
            Token::Minus => {
                self.advance();
                match self.advance() {
                    Token::Int(n) => MatchPat::Int(-n),
                    t => return Err(format!("Ожидалось целое число после '-', получено {:?}", t)),
                }
            }
            // `_` wildcard or `EnumName.Variant`  (from Rust / Swift)
            Token::Ident(s) if s == "_" => {
                self.advance();
                MatchPat::Wildcard
            }
            Token::Ident(first) if matches!(self.peek2(), Token::Dot)
                && matches!(self.peek3(), Token::Ident(_)) =>
            {
                // EnumName.Variant pattern
                self.advance(); // EnumName
                self.advance(); // '.'
                let variant = self.expect_ident()?;
                MatchPat::EnumVariant(first, variant)
            }
            t => return Err(format!(
                "Ожидался образец match (целое, _, EnumName.Variant), получено {:?}", t
            )),
        };
        self.expect(&Token::FatArrow)?;
        let body = self.parse_block_stmts()?;
        Ok(MatchArm { pat, body })
    }

    // ── Expressions ────────────────────────────────────────────────────────
    //
    //  Precedence (low → high):
    //    pipe       :  |>            (from Elixir / F#)
    //    ternary    :  ? :           (from C / Java)
    //    or_expr    :  ||
    //    and_expr   :  &&
    //    cmp_expr   :  == != < <= > >=
    //    add_expr   :  + -
    //    mul_expr   :  * / %
    //    unary      :  - !
    //    power      :  **            (from Python) — right-associative
    //    postfix    :  expr.field / expr.method(args) / expr[idx]
    //    call_base  :  name(args) / StructName { ... } / new ClassName(...)
    //    primary    :  literal | ident | self | (expr) | [arr] | $"..."

    pub fn parse_expr(&mut self) -> Result<Expr, String> { self.parse_pipe() }

    /// `expr |> func`  or  `expr |> func(extra_args)`  (from Elixir / F#)
    /// Desugars: `x |> f` → `f(x)`, `x |> f(a, b)` → `f(x, a, b)`
    fn parse_pipe(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_ternary()?;
        while self.eat(&Token::PipeGt) {
            match self.peek().clone() {
                Token::Ident(name) => {
                    self.advance();
                    if self.eat(&Token::LParen) {
                        let mut extra = self.parse_arg_list()?;
                        self.expect(&Token::RParen)?;
                        let mut args = vec![lhs];
                        args.append(&mut extra);
                        lhs = Expr::Call { name, args };
                    } else {
                        lhs = Expr::Call { name, args: vec![lhs] };
                    }
                }
                t => return Err(format!(
                    "Ожидалось имя функции после '|>', получено {:?}", t
                )),
            }
        }
        Ok(lhs)
    }

    /// `cond ? then : els`  (from C / Java)
    fn parse_ternary(&mut self) -> Result<Expr, String> {
        let cond = self.parse_or()?;
        if self.eat(&Token::Question) {
            let then = self.parse_or()?;
            self.expect(&Token::Colon)?;
            let els = self.parse_ternary()?; // right-associative
            return Ok(Expr::Ternary {
                cond: Box::new(cond),
                then: Box::new(then),
                els:  Box::new(els),
            });
        }
        Ok(cond)
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_and()?;
        while self.eat(&Token::OrOr) {
            let rhs = self.parse_and()?;
            lhs = Expr::Binary(Box::new(lhs), BinOp::Or, Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_cmp()?;
        while self.eat(&Token::AndAnd) {
            let rhs = self.parse_cmp()?;
            lhs = Expr::Binary(Box::new(lhs), BinOp::And, Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_cmp(&mut self) -> Result<Expr, String> {
        let lhs = self.parse_add()?;
        let op = match self.peek() {
            Token::EqEq   => BinOp::Eq,
            Token::BangEq => BinOp::Ne,
            Token::Lt     => BinOp::Lt,
            Token::LtEq   => BinOp::Le,
            Token::Gt     => BinOp::Gt,
            Token::GtEq   => BinOp::Ge,
            _ => return Ok(lhs),
        };
        self.advance();
        let rhs = self.parse_add()?;
        Ok(Expr::Binary(Box::new(lhs), op, Box::new(rhs)))
    }

    fn parse_add(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_mul()?;
        loop {
            let op = match self.peek() {
                Token::Plus  => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_mul()?;
            lhs = Expr::Binary(Box::new(lhs), op, Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_mul(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Token::Star    => BinOp::Mul,
                Token::Slash   => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_unary()?;
            lhs = Expr::Binary(Box::new(lhs), op, Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        match self.peek().clone() {
            Token::Minus => {
                self.advance();
                Ok(Expr::Unary(UnaryOp::Neg, Box::new(self.parse_unary()?)))
            }
            Token::Bang => {
                self.advance();
                Ok(Expr::Unary(UnaryOp::Not, Box::new(self.parse_unary()?)))
            }
            // &expr — address-of operator (low-level pointer)
            Token::Amp => {
                self.advance();
                Ok(Expr::AddrOf(Box::new(self.parse_unary()?)))
            }
            // *expr — dereference operator (load i64 from address)
            Token::Star => {
                self.advance();
                Ok(Expr::Deref(Box::new(self.parse_unary()?)))
            }
            _ => self.parse_power(),
        }
    }

    /// `base ** exp`  (from Python) — right-associative, higher than unary
    fn parse_power(&mut self) -> Result<Expr, String> {
        let base = self.parse_postfix()?;
        if self.eat(&Token::StarStar) {
            let exp = self.parse_unary()?; // right-assoc: 2**3**2 = 2**(3**2)
            return Ok(Expr::Binary(Box::new(base), BinOp::Pow, Box::new(exp)));
        }
        Ok(base)
    }

    /// Parse a base expression then consume any dot-chains and index accesses.
    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_call_base()?;
        loop {
            if self.check(&Token::Dot) {
                self.advance(); // consume '.'
                let member = self.expect_ident()?;
                if self.eat(&Token::LParen) {
                    let args = self.parse_arg_list()?;
                    self.expect(&Token::RParen)?;
                    expr = Expr::MethodCall {
                        obj:    Box::new(expr),
                        method: member,
                        args,
                    };
                } else {
                    expr = Expr::FieldAccess {
                        obj:   Box::new(expr),
                        field: member,
                    };
                }
            } else if self.check(&Token::LBracket) {
                // array indexing: expr[idx]  (from Python / JS)
                self.advance(); // '['
                let idx = self.parse_expr()?;
                self.expect(&Token::RBracket)?;
                expr = Expr::Index { arr: Box::new(expr), idx: Box::new(idx) };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    /// Parse a function call, struct literal, constructor call, or primary.
    ///
    /// Syntax:
    ///   new ClassName(args)       → ConstructorCall  (Java/C# style)
    ///   StructName { field: expr } → StructLit        (Go style, no `new`)
    ///   name(args)                → Call
    ///   readInt()                 → Input  (built-in)
    ///   readFloat()               → InputFloat (built-in)
    fn parse_call_base(&mut self) -> Result<Expr, String> {
        // `new ClassName(args)` — constructor call
        if self.eat(&Token::New) {
            let name = self.expect_ident()?;
            self.expect(&Token::LParen)?;
            let args = self.parse_arg_list()?;
            self.expect(&Token::RParen)?;
            return Ok(Expr::ConstructorCall { class: name, args });
        }

        if let Token::Ident(name) = self.peek().clone() {
            // `readInt()` → Expr::Input
            if name == "readInt" && matches!(self.peek2(), Token::LParen) {
                self.advance(); // 'readInt'
                self.advance(); // '('
                self.expect(&Token::RParen)?;
                return Ok(Expr::Input);
            }
            // `readFloat()` → Expr::InputFloat
            if name == "readFloat" && matches!(self.peek2(), Token::LParen) {
                self.advance(); // 'readFloat'
                self.advance(); // '('
                self.expect(&Token::RParen)?;
                return Ok(Expr::InputFloat);
            }
            // `cstr("literal")` → Expr::CStr  — address of null-terminated C string
            if name == "cstr" && matches!(self.peek2(), Token::LParen) {
                self.advance(); // 'cstr'
                self.advance(); // '('
                let s = match self.advance() {
                    Token::Str(s) => s,
                    t => return Err(format!("cstr() ожидает строковый литерал, получено {:?}", t)),
                };
                self.expect(&Token::RParen)?;
                return Ok(Expr::CStr(s));
            }
            // `name(args)` → Call
            if matches!(self.peek2(), Token::LParen) {
                self.advance(); // ident
                self.advance(); // (
                let args = self.parse_arg_list()?;
                self.expect(&Token::RParen)?;
                return Ok(Expr::Call { name, args });
            }
            // `StructName { field: expr, ... }` → StructLit
            // Only when { is followed by `ident:` (field pair) or `}` (empty struct),
            // so that `match score { 1 => ... }` is NOT misread as a struct literal.
            if matches!(self.peek2(), Token::LBrace) && self.looks_like_struct_lit() {
                self.advance(); // ident
                return self.parse_struct_lit_body(name);
            }
        }

        self.parse_primary()
    }

    /// True when `tokens[pos+1]` is `{` and the char after it is `}` (empty struct)
    /// or `ident :` (field-value pair). Prevents `match expr {` from being misread.
    fn looks_like_struct_lit(&self) -> bool {
        // pos   → Ident  (already confirmed by caller)
        // pos+1 → LBrace (already confirmed by caller)
        // pos+2 → first token inside the brace
        // pos+3 → token after that
        matches!(self.tokens.get(self.pos + 2), Some(Token::RBrace))
            || (matches!(self.tokens.get(self.pos + 2), Some(Token::Ident(_)))
                && matches!(self.tokens.get(self.pos + 3), Some(Token::Colon)))
    }

    fn parse_struct_lit_body(&mut self, name: String) -> Result<Expr, String> {
        self.expect(&Token::LBrace)?;
        let mut fields = Vec::new();
        while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
            let fname = self.expect_ident()?;
            self.expect(&Token::Colon)?;
            let fval = self.parse_expr()?;
            fields.push((fname, fval));
            self.eat(&Token::Comma);
        }
        self.expect(&Token::RBrace)?;
        Ok(Expr::StructLit { name, fields })
    }

    fn parse_arg_list(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        if self.check(&Token::RParen) { return Ok(args); }
        args.push(self.parse_expr()?);
        while self.eat(&Token::Comma) {
            args.push(self.parse_expr()?);
        }
        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.advance() {
            Token::Int(n)   => Ok(Expr::Number(n)),
            Token::Float(f) => Ok(Expr::Float(f)),
            Token::Str(s)   => Ok(Expr::Str(s)),
            // `$"Hello {name}!"` — interpolated string  (from C# / Kotlin)
            Token::InterpolStr(parts) => Ok(Expr::Interpolated(parts)),
            Token::True     => Ok(Expr::Number(1)),
            Token::False    => Ok(Expr::Number(0)),
            Token::Ident(n) => Ok(Expr::Ident(n)),
            Token::SelfKw   => Ok(Expr::Ident("self".into())),
            Token::LParen   => {
                let e = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(e)
            }
            // `[expr, ...]` — array literal  (from Python / JS)
            Token::LBracket => {
                let mut elems = Vec::new();
                if !self.check(&Token::RBracket) {
                    elems.push(self.parse_expr()?);
                    while self.eat(&Token::Comma) {
                        if self.check(&Token::RBracket) { break; } // trailing comma
                        elems.push(self.parse_expr()?);
                    }
                }
                self.expect(&Token::RBracket)?;
                Ok(Expr::ArrayLit(elems))
            }
            t => Err(format!("Ожидалось выражение, получено {:?}", t)),
        }
    }
}
