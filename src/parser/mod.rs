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
        else { Err(format!("Expected {:?}, got {:?}", tok, got)) }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.advance() {
            Token::Ident(s) => Ok(s),
            t => Err(format!("Expected identifier, got {:?}", t)),
        }
    }

    /// Like `expect_ident` but also accepts keyword `new` as a method name.
    fn expect_method_name(&mut self) -> Result<String, String> {
        match self.advance() {
            Token::Ident(s) => Ok(s),
            Token::New      => Ok("new".to_string()),
            t => Err(format!("Expected method name, got {:?}", t)),
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
        // Consume optional annotations (@name) before any declaration
        if self.check(&Token::At) {
            return self.parse_annotation();
        }
        match self.peek() {
            Token::Func   => self.parse_fn_decl(),
            Token::Struct => self.parse_struct_decl(),
            Token::Impl   => self.parse_impl_or_trait_impl(),
            Token::Class  => self.parse_class_decl(),
            Token::Enum   => self.parse_enum_decl(),
            Token::Const  => self.parse_const(),
            Token::Import => self.parse_import(),
            Token::Extern => self.parse_extern_fn(),
            Token::Trait  => self.parse_trait_decl(),
            t => Err(format!(
                "Expected top-level declaration, got {:?}", t.clone()
            )),
        }
    }

    fn parse_annotation(&mut self) -> Result<Stmt, String> {
        self.expect(&Token::At)?;
        let name = self.expect_ident()?;
        // Optionally consume parenthesised args: @name(...)
        if self.eat(&Token::LParen) {
            while !self.check(&Token::RParen) && !self.check(&Token::Eof) {
                self.advance();
            }
            self.eat(&Token::RParen);
        }
        Ok(Stmt::Annotation { name })
    }

    fn parse_import(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'import'
        let path = match self.advance() {
            Token::Str(s) => s,
            t => return Err(format!("import expects a string, got {:?}", t)),
        };
        self.eat(&Token::Semicolon);
        Ok(Stmt::Import { path })
    }

    fn parse_extern_fn(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'extern'
        self.expect(&Token::Func)?;
        let name = self.expect_ident()?;
        self.expect(&Token::LParen)?;

        let mut params: usize = 0;
        let mut variadic = false;

        if !self.check(&Token::RParen) {
            loop {
                if self.check(&Token::DotDot) {
                    self.advance();
                    if self.eat(&Token::Dot) {
                        variadic = true;
                        break;
                    }
                    return Err("Expected '...' for variadic".into());
                }
                self.expect_ident()?;
                if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
                params += 1;
                if !self.eat(&Token::Comma) { break; }
            }
        }

        self.expect(&Token::RParen)?;
        if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
        // also accept ->  as return-type annotation  (Rust / Swift style)
        if self.eat(&Token::Arrow) { self.skip_type_annotation()?; }
        self.eat(&Token::Semicolon);
        Ok(Stmt::ExternFn { name, params, variadic })
    }

    fn parse_fn_decl(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'func'
        let name = self.expect_ident()?;
        self.expect(&Token::LParen)?;
        let params = self.parse_param_list_with_defaults()?;
        self.expect(&Token::RParen)?;
        // accept `:` or `->` as return-type annotation  (Rust / Swift)
        if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
        if self.eat(&Token::Arrow) { self.skip_type_annotation()?; }
        let body = self.parse_block_stmts()?;
        Ok(Stmt::FnDecl { name, params, body })
    }

    /// Parse parameter list with optional default values:
    /// `(a: int, b: int = 0, c: int = 1)`
    fn parse_param_list_with_defaults(&mut self) -> Result<Vec<Param>, String> {
        let mut params = Vec::new();
        if self.check(&Token::RParen) { return Ok(params); }
        params.push(self.parse_one_param()?);
        while self.eat(&Token::Comma) {
            params.push(self.parse_one_param()?);
        }
        Ok(params)
    }

    fn parse_one_param(&mut self) -> Result<Param, String> {
        let name = self.expect_ident()?;
        if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
        // default value: `= expr`
        let default = if self.eat(&Token::Assign) {
            Some(self.parse_expr()?)
        } else {
            None
        };
        Ok((name, default))
    }

    /// Skip over a type name token (int / float / identifier).
    fn skip_type_annotation(&mut self) -> Result<(), String> {
        match self.peek().clone() {
            Token::Ident(_) => { self.advance(); Ok(()) }
            t => Err(format!("Expected type name, got {:?}", t)),
        }
    }

    // ── Trait declaration ──────────────────────────────────────────────────

    /// `trait Name { [pub] func method(self [, params]); ... }`
    fn parse_trait_decl(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'trait'
        let name = self.expect_ident()?;
        self.expect(&Token::LBrace)?;
        let mut methods = Vec::new();
        while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
            self.eat(&Token::Pub);
            self.eat(&Token::Private);
            self.expect(&Token::Func)?;
            let mname = self.expect_method_name()?;
            self.expect(&Token::LParen)?;
            let mut params = Vec::new();
            if !self.check(&Token::RParen) {
                // consume self / param list
                if self.check(&Token::SelfKw) { self.advance(); }
                else { params.push(self.expect_ident()?); }
                while self.eat(&Token::Comma) {
                    if self.eat(&Token::Colon) { self.skip_type_annotation()?; continue; }
                    params.push(self.expect_ident()?);
                    if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
                }
            }
            self.expect(&Token::RParen)?;
            if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
            if self.eat(&Token::Arrow) { self.skip_type_annotation()?; }
            self.eat(&Token::Semicolon);
            methods.push((mname, params));
        }
        self.expect(&Token::RBrace)?;
        Ok(Stmt::TraitDecl { name, methods })
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
            t => Err(format!("Expected field type, got {:?}", t)),
        }
    }

    /// Dispatch: `impl Name { }` or `impl Trait for Type { }` or `impl Trait { }` (operator overloading)
    fn parse_impl_or_trait_impl(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'impl'
        let first = self.expect_ident()?;

        // `impl Trait for Type { }` — trait implementation
        if self.eat(&Token::For) {
            let for_type = self.expect_ident()?;
            self.expect(&Token::LBrace)?;
            let mut methods = Vec::new();
            while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
                let access = self.parse_access_modifier();
                let is_static = self.eat(&Token::Static);
                let mut m = self.parse_method_decl()?;
                m.access    = access;
                m.is_static = is_static;
                methods.push(m);
            }
            self.expect(&Token::RBrace)?;
            return Ok(Stmt::ImplTrait { trait_name: first, for_type, methods });
        }

        // `impl Name { }` — inherent impl block
        self.expect(&Token::LBrace)?;
        let mut methods = Vec::new();
        while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
            let access = self.parse_access_modifier();
            let is_static = self.eat(&Token::Static);
            let mut m = self.parse_method_decl()?;
            m.access    = access;
            m.is_static = is_static;
            methods.push(m);
        }
        self.expect(&Token::RBrace)?;
        Ok(Stmt::ImplDecl { struct_name: first, methods })
    }

    /// `class Name [extends Parent] { ... }`
    fn parse_class_decl(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'class'
        let name = self.expect_ident()?;

        let parent = if self.eat(&Token::Extends) {
            Some(self.expect_ident()?)
        } else {
            None
        };

        self.expect(&Token::LBrace)?;

        let mut fields  = Vec::new();
        let mut methods = Vec::new();

        while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
            let access = self.parse_access_modifier();
            let is_static = self.eat(&Token::Static);

            if self.check(&Token::Func) || self.check(&Token::Init) {
                let mut m = self.parse_method_decl()?;
                m.access    = access;
                m.is_static = is_static;
                methods.push(m);
            } else {
                let fname = self.expect_ident()?;
                self.expect(&Token::Colon)?;
                let ftype = self.parse_field_type()?;
                self.eat(&Token::Comma);
                fields.push(FieldDecl { name: fname, ty: ftype, access });
            }
        }

        self.expect(&Token::RBrace)?;
        Ok(Stmt::ClassDecl { name, parent, fields, methods })
    }

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

    fn parse_const(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'const'
        let name = self.expect_ident()?;
        if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
        self.expect(&Token::Assign)?;
        let expr = self.parse_expr()?;
        self.expect(&Token::Semicolon)?;
        Ok(Stmt::Const { name, expr })
    }

    fn parse_access_modifier(&mut self) -> Access {
        if self.eat(&Token::Pub)          { Access::Public  }
        else if self.eat(&Token::Private) { Access::Private }
        else                              { Access::Public  }
    }

    fn parse_method_decl(&mut self) -> Result<MethodDecl, String> {
        let is_init = self.check(&Token::Init);
        if is_init { self.advance(); } else { self.expect(&Token::Func)?; }

        let name = if is_init { "new".to_string() } else { self.expect_method_name()? };

        self.expect(&Token::LParen)?;

        let mut has_self = is_init;
        let mut params: Vec<Param> = Vec::new();

        if is_init {
            if !self.check(&Token::RParen) {
                params.push(self.parse_one_param()?);
                while self.eat(&Token::Comma) {
                    params.push(self.parse_one_param()?);
                }
            }
        } else {
            if !self.check(&Token::RParen) {
                if self.check(&Token::SelfKw) {
                    self.advance();
                    has_self = true;
                    if self.eat(&Token::Comma) {
                        params.push(self.parse_one_param()?);
                        while self.eat(&Token::Comma) {
                            params.push(self.parse_one_param()?);
                        }
                    }
                } else {
                    params.push(self.parse_one_param()?);
                    while self.eat(&Token::Comma) {
                        params.push(self.parse_one_param()?);
                    }
                }
            }
        }

        self.expect(&Token::RParen)?;
        if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
        if self.eat(&Token::Arrow) { self.skip_type_annotation()?; }
        let body = self.parse_block_stmts()?;
        Ok(MethodDecl { name, params, has_self, is_static: false, body, access: Access::Public })
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
        // annotations inside function bodies are silently consumed
        if self.check(&Token::At) {
            return self.parse_annotation();
        }

        if self.is_field_assign_stmt() {
            return self.parse_field_assign_stmt();
        }

        match self.peek().clone() {
            Token::Var      => self.parse_var(),
            Token::Const    => self.parse_const(),
            Token::If       => self.parse_if(),
            Token::Unless   => self.parse_unless(),
            Token::While    => self.parse_while(),
            Token::Do       => self.parse_do_while(),
            Token::For      => self.parse_for(),
            Token::Loop     => self.parse_loop(),
            Token::Repeat   => self.parse_repeat(),
            Token::Defer    => self.parse_defer_stmt(),
            Token::Return   => self.parse_return(),
            Token::Println  => self.parse_println(),
            Token::Match    => self.parse_match_stmt(),
            Token::Break    => { self.advance(); self.expect(&Token::Semicolon)?; Ok(Stmt::Break) }
            Token::Continue => { self.advance(); self.expect(&Token::Semicolon)?; Ok(Stmt::Continue) }
            Token::LBrace => {
                let body = self.parse_block_stmts()?;
                Ok(Stmt::Block(body))
            }
            // compound assignment:  ident op= expr ;
            Token::Ident(_) if matches!(
                self.peek2(),
                Token::PlusAssign | Token::MinusAssign |
                Token::StarAssign | Token::SlashAssign |
                Token::PercentAssign | Token::CaretAssign
            ) => self.parse_compound_assign(),
            // simple assignment:  ident = expr ;
            Token::Ident(_) if matches!(self.peek2(), Token::Assign) => {
                self.parse_assign()
            }
            // expression statement (may be index assignment: arr[i] = val;)
            _ => {
                let e = self.parse_expr()?;
                if self.eat(&Token::Assign) {
                    let val = self.parse_expr()?;
                    self.expect(&Token::Semicolon)?;
                    match e {
                        Expr::Index { arr, idx } =>
                            return Ok(Stmt::IndexAssign { arr, idx, val }),
                        _ => return Err(
                            "Invalid left-hand side of assignment".into()
                        ),
                    }
                }
                self.expect(&Token::Semicolon)?;
                Ok(Stmt::Expr(e))
            }
        }
    }

    fn parse_var(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'var'
        // Tuple destructuring: var (a, b) = expr;
        if self.check(&Token::LParen) {
            self.advance(); // '('
            let mut names = Vec::new();
            names.push(self.expect_ident()?);
            while self.eat(&Token::Comma) {
                names.push(self.expect_ident()?);
            }
            self.expect(&Token::RParen)?;
            self.expect(&Token::Assign)?;
            let expr = self.parse_expr()?;
            self.expect(&Token::Semicolon)?;
            return Ok(Stmt::LetTuple { names, expr });
        }
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

    fn parse_compound_assign(&mut self) -> Result<Stmt, String> {
        let name = self.expect_ident()?;
        let op = match self.advance() {
            Token::PlusAssign    => BinOp::Add,
            Token::MinusAssign   => BinOp::Sub,
            Token::StarAssign    => BinOp::Mul,
            Token::SlashAssign   => BinOp::Div,
            Token::PercentAssign => BinOp::Mod,
            Token::CaretAssign   => BinOp::Xor,
            t => return Err(format!("Expected assignment operator, got {:?}", t)),
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
            t => return Err(format!("Expected identifier or 'self', got {:?}", t)),
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

    /// `for i in start..end` / `for i in start..=end` / `for x in array`
    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'for'

        // Check for array iteration: `for x in arr { }`
        // If after `ident in expr` there is no `..` or `..=`, it's ForIn.
        let saved_pos = self.pos;

        let var = self.expect_ident()?;
        self.expect(&Token::In)?;

        // Parse the expression after 'in'
        let iter_expr = self.parse_add()?; // parse up to addition level to avoid consuming range ops

        // If next token is `..` or `..=` → range for; else → ForIn
        if self.check(&Token::DotDot) || self.check(&Token::DotDotEq) {
            // Restore full range parsing
            let inclusive = if self.eat(&Token::DotDotEq) { true } else { self.eat(&Token::DotDot); false };
            let to = self.parse_expr()?;

            // Check for multi-range: `for i in 0..3, j in 0..5`
            if self.eat(&Token::Comma) {
                let mut ranges = vec![(var, iter_expr, to, inclusive)];
                loop {
                    let v2 = self.expect_ident()?;
                    self.expect(&Token::In)?;
                    let f2 = self.parse_expr()?;
                    let inc2 = if self.eat(&Token::DotDotEq) { true } else { self.expect(&Token::DotDot)?; false };
                    let t2 = self.parse_expr()?;
                    ranges.push((v2, f2, t2, inc2));
                    if !self.eat(&Token::Comma) { break; }
                }
                let body_stmts = self.parse_block_stmts()?;
                let innermost = Stmt::Block(body_stmts);
                let result = ranges.into_iter().rev().fold(innermost, |inner, (v, f, t, inc)| {
                    Stmt::For { var: v, from: f, to: t, inclusive: inc, body: Box::new(inner) }
                });
                return Ok(result);
            }

            let body_stmts = self.parse_block_stmts()?;
            return Ok(Stmt::For {
                var,
                from: iter_expr,
                to,
                inclusive,
                body: Box::new(Stmt::Block(body_stmts)),
            });
        }

        // ForIn — array/collection iteration
        let _ = saved_pos; // pos already advanced correctly
        let body_stmts = self.parse_block_stmts()?;
        Ok(Stmt::ForIn {
            var,
            iter: iter_expr,
            body: Box::new(Stmt::Block(body_stmts)),
        })
    }

    fn parse_loop(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'loop'
        let body = self.parse_block_stmts()?;
        Ok(Stmt::Loop { body: Box::new(Stmt::Block(body)) })
    }

    /// `repeat N { body }` — desugars to `for __ri in 0..N { body }`
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

    fn parse_println(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'println'
        self.expect(&Token::LParen)?;
        let e = self.parse_expr()?;
        self.expect(&Token::RParen)?;
        self.expect(&Token::Semicolon)?;
        Ok(Stmt::Print(e))
    }

    fn parse_match_stmt(&mut self) -> Result<Stmt, String> {
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
        let pat = self.parse_match_pat()?;
        self.expect(&Token::FatArrow)?;
        let body = self.parse_block_stmts()?;
        Ok(MatchArm { pat, body })
    }

    fn parse_match_pat(&mut self) -> Result<MatchPat, String> {
        match self.peek().clone() {
            Token::Int(n) => { self.advance(); Ok(MatchPat::Int(n)) }
            Token::Minus  => {
                self.advance();
                match self.advance() {
                    Token::Int(n) => Ok(MatchPat::Int(-n)),
                    t => Err(format!("Expected integer after '-', got {:?}", t)),
                }
            }
            Token::Ident(s) if s == "_" => { self.advance(); Ok(MatchPat::Wildcard) }
            Token::Ident(first) if matches!(self.peek2(), Token::Dot)
                && matches!(self.peek3(), Token::Ident(_)) =>
            {
                self.advance();
                self.advance(); // '.'
                let variant = self.expect_ident()?;
                Ok(MatchPat::EnumVariant(first, variant))
            }
            t => Err(format!("Expected match pattern, got {:?}", t)),
        }
    }

    // ── Expressions ────────────────────────────────────────────────────────
    //
    //  Precedence (low → high):
    //    pipe       :  |>             (Elixir / F#)
    //    ternary    :  ? :            (C / Java)
    //    or_expr    :  ||
    //    and_expr   :  &&
    //    xor_expr   :  ^             (C / Java) — NEW
    //    cmp_expr   :  == != < <= > >=
    //    add_expr   :  + -
    //    mul_expr   :  * / %
    //    unary      :  - ! ~         (~ is bitwise NOT — NEW)
    //    power      :  **            (Python) — right-associative
    //    postfix    :  expr.field / expr.method(args) / expr[idx] / expr::method(args)
    //    call_base  :  name(args) / Type::method(args) / StructName { ... } / new ...
    //    primary    :  literal | ident | self | (expr) | [arr] | $"..." | |x| expr | match

    pub fn parse_expr(&mut self) -> Result<Expr, String> { self.parse_pipe() }

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
                t => return Err(format!("Expected function name after '|>', got {:?}", t)),
            }
        }
        Ok(lhs)
    }

    fn parse_ternary(&mut self) -> Result<Expr, String> {
        let cond = self.parse_or()?;
        if self.eat(&Token::Question) {
            let then = self.parse_or()?;
            self.expect(&Token::Colon)?;
            let els = self.parse_ternary()?;
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
        let mut lhs = self.parse_xor()?;
        while self.eat(&Token::AndAnd) {
            let rhs = self.parse_xor()?;
            lhs = Expr::Binary(Box::new(lhs), BinOp::And, Box::new(rhs));
        }
        Ok(lhs)
    }

    /// `a ^ b` — XOR operator  (C / Java)
    fn parse_xor(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_cmp()?;
        while self.eat(&Token::Caret) {
            let rhs = self.parse_cmp()?;
            lhs = Expr::Binary(Box::new(lhs), BinOp::Xor, Box::new(rhs));
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
            // ~expr — bitwise NOT  (C / Java)
            Token::Tilde => {
                self.advance();
                Ok(Expr::Unary(UnaryOp::BitNot, Box::new(self.parse_unary()?)))
            }
            // &expr — address-of
            Token::Amp => {
                self.advance();
                Ok(Expr::AddrOf(Box::new(self.parse_unary()?)))
            }
            // *expr — dereference
            Token::Star => {
                self.advance();
                Ok(Expr::Deref(Box::new(self.parse_unary()?)))
            }
            _ => self.parse_power(),
        }
    }

    fn parse_power(&mut self) -> Result<Expr, String> {
        let base = self.parse_postfix()?;
        if self.eat(&Token::StarStar) {
            let exp = self.parse_unary()?;
            return Ok(Expr::Binary(Box::new(base), BinOp::Pow, Box::new(exp)));
        }
        Ok(base)
    }

    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_call_base()?;
        loop {
            if self.check(&Token::Dot) {
                self.advance();
                let member = self.expect_ident()?;
                if self.eat(&Token::LParen) {
                    let args = self.parse_arg_list()?;
                    self.expect(&Token::RParen)?;
                    expr = Expr::MethodCall { obj: Box::new(expr), method: member, args };
                } else {
                    expr = Expr::FieldAccess { obj: Box::new(expr), field: member };
                }
            } else if self.check(&Token::LBracket) {
                self.advance();
                let idx = self.parse_expr()?;
                self.expect(&Token::RBracket)?;
                expr = Expr::Index { arr: Box::new(expr), idx: Box::new(idx) };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_call_base(&mut self) -> Result<Expr, String> {
        // `new ClassName(args)` — constructor call
        if self.eat(&Token::New) {
            let name = self.expect_ident()?;
            self.expect(&Token::LParen)?;
            let args = self.parse_arg_list()?;
            self.expect(&Token::RParen)?;
            return Ok(Expr::ConstructorCall { class: name, args });
        }

        // `|params| expr` — lambda  (Rust / Python)
        if self.check(&Token::Pipe) {
            return self.parse_lambda();
        }

        // `match expr { pat => val, ... }` — match as expression
        if self.check(&Token::Match) {
            return self.parse_match_expr();
        }

        if let Token::Ident(name) = self.peek().clone() {
            // `Type::method(args)` — static call  (C++ / Rust)
            if matches!(self.peek2(), Token::ColonColon) {
                self.advance(); // type name
                self.advance(); // ::
                let method = self.expect_ident()?;
                self.expect(&Token::LParen)?;
                let args = self.parse_arg_list()?;
                self.expect(&Token::RParen)?;
                return Ok(Expr::StaticCall { type_name: name, method, args });
            }

            // `readInt()` → Expr::Input
            if name == "readInt" && matches!(self.peek2(), Token::LParen) {
                self.advance(); self.advance();
                self.expect(&Token::RParen)?;
                return Ok(Expr::Input);
            }
            // `readFloat()` → Expr::InputFloat
            if name == "readFloat" && matches!(self.peek2(), Token::LParen) {
                self.advance(); self.advance();
                self.expect(&Token::RParen)?;
                return Ok(Expr::InputFloat);
            }
            // `cstr("literal")` → Expr::CStr
            if name == "cstr" && matches!(self.peek2(), Token::LParen) {
                self.advance(); self.advance();
                let s = match self.advance() {
                    Token::Str(s) => s,
                    t => return Err(format!("cstr() expects a string literal, got {:?}", t)),
                };
                self.expect(&Token::RParen)?;
                return Ok(Expr::CStr(s));
            }
            // `name(args)` → Call
            if matches!(self.peek2(), Token::LParen) {
                self.advance(); self.advance();
                let args = self.parse_arg_list()?;
                self.expect(&Token::RParen)?;
                return Ok(Expr::Call { name, args });
            }
            // `StructName { field: expr }` → StructLit
            if matches!(self.peek2(), Token::LBrace) && self.looks_like_struct_lit() {
                self.advance();
                return self.parse_struct_lit_body(name);
            }
        }

        self.parse_primary()
    }

    /// `|params| expr`  — lambda expression  (Rust / Python)
    fn parse_lambda(&mut self) -> Result<Expr, String> {
        self.expect(&Token::Pipe)?; // opening |
        let mut params = Vec::new();
        while !self.check(&Token::Pipe) && !self.check(&Token::Eof) {
            params.push(self.expect_ident()?);
            if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
            if !self.eat(&Token::Comma) { break; }
        }
        self.expect(&Token::Pipe)?; // closing |
        let body = self.parse_expr()?;
        Ok(Expr::Lambda { params, body: Box::new(body) })
    }

    /// `match expr { pat => expr_val, ... }` — match as expression
    fn parse_match_expr(&mut self) -> Result<Expr, String> {
        self.advance(); // 'match'
        let expr = self.parse_add()?; // avoid consuming { as part of expr
        self.expect(&Token::LBrace)?;
        let mut arms = Vec::new();
        while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
            let pat = self.parse_match_pat()?;
            self.expect(&Token::FatArrow)?;
            // Expression arm: `pat => expr,`  (no braces)
            let val = self.parse_expr()?;
            self.eat(&Token::Comma);
            arms.push(MatchArmExpr { pat, val });
        }
        self.expect(&Token::RBrace)?;
        Ok(Expr::MatchExpr { expr: Box::new(expr), arms })
    }

    fn looks_like_struct_lit(&self) -> bool {
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
            Token::InterpolStr(parts) => Ok(Expr::Interpolated(parts)),
            Token::True     => Ok(Expr::Number(1)),
            Token::False    => Ok(Expr::Number(0)),
            Token::Ident(n) => Ok(Expr::Ident(n)),
            Token::SelfKw   => Ok(Expr::Ident("self".into())),
            Token::LParen   => {
                let first = self.parse_expr()?;
                // Tuple: (a, b, ...)
                if self.eat(&Token::Comma) {
                    let mut elems = vec![first];
                    elems.push(self.parse_expr()?);
                    while self.eat(&Token::Comma) {
                        if self.check(&Token::RParen) { break; }
                        elems.push(self.parse_expr()?);
                    }
                    self.expect(&Token::RParen)?;
                    return Ok(Expr::Tuple(elems));
                }
                self.expect(&Token::RParen)?;
                Ok(first)
            }
            // `[expr, ...]` — array literal  (Python / JS)
            Token::LBracket => {
                let mut elems = Vec::new();
                if !self.check(&Token::RBracket) {
                    elems.push(self.parse_expr()?);
                    while self.eat(&Token::Comma) {
                        if self.check(&Token::RBracket) { break; }
                        elems.push(self.parse_expr()?);
                    }
                }
                self.expect(&Token::RBracket)?;
                Ok(Expr::ArrayLit(elems))
            }
            t => Err(format!("Expected expression, got {:?}", t)),
        }
    }
}
