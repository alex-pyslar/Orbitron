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
            t => Err(format!(
                "Ожидалось 'func', 'struct', 'impl' или 'class' на верхнем уровне, получено {:?}",
                t.clone()
            )),
        }
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
            Token::If       => self.parse_if(),
            Token::While    => self.parse_while(),
            Token::Do       => self.parse_do_while(),
            Token::For      => self.parse_for(),
            Token::Loop     => self.parse_loop(),
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
            // expression statement
            _ => {
                let e = self.parse_expr()?;
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
            Token::Ident(s) if s == "_" => {
                self.advance();
                MatchPat::Wildcard
            }
            t => return Err(format!("Ожидался образец match (целое или _), получено {:?}", t)),
        };
        self.expect(&Token::FatArrow)?;
        let body = self.parse_block_stmts()?;
        Ok(MatchArm { pat, body })
    }

    // ── Expressions ────────────────────────────────────────────────────────
    //
    //  Precedence (low → high):
    //    or_expr    :  ||
    //    and_expr   :  &&
    //    cmp_expr   :  == != < <= > >=
    //    add_expr   :  + -
    //    mul_expr   :  * / %
    //    unary      :  - !
    //    postfix    :  expr.field / expr.method(args)
    //    call_base  :  name(args) / StructName { ... } / new ClassName(...)
    //    primary    :  literal | ident | self | (expr)

    pub fn parse_expr(&mut self) -> Result<Expr, String> { self.parse_or() }

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
            _ => self.parse_postfix(),
        }
    }

    /// Parse a base expression then consume any dot-chains.
    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_call_base()?;
        while self.check(&Token::Dot) {
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
            Token::True     => Ok(Expr::Number(1)),
            Token::False    => Ok(Expr::Number(0)),
            Token::Ident(n) => Ok(Expr::Ident(n)),
            Token::SelfKw   => Ok(Expr::Ident("self".into())),
            Token::LParen   => {
                let e = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(e)
            }
            t => Err(format!("Ожидалось выражение, получено {:?}", t)),
        }
    }
}
