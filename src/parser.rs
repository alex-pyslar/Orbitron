use crate::ast::*;
use crate::lexer::Token;

// ── Parser ─────────────────────────────────────────────────────────────────

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

    fn peek_at(&self, offset: usize) -> &Token {
        self.tokens.get(self.pos + offset).unwrap_or(&Token::Eof)
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
            Token::Fn     => self.parse_fn_decl(),
            Token::Main   => self.parse_main_decl(),
            Token::Struct => self.parse_struct_decl(),
            Token::Impl   => self.parse_impl_decl(),
            t => Err(format!(
                "Expected 'fn', 'main', 'struct', or 'impl' at top level, got {:?}",
                t.clone()
            )),
        }
    }

    fn parse_fn_decl(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'fn'
        let name = self.expect_ident()?;
        self.expect(&Token::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(&Token::RParen)?;
        if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
        let body = self.parse_block_stmts()?;
        Ok(Stmt::FnDecl { name, params, body })
    }

    fn parse_main_decl(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'main'
        let body = self.parse_block_stmts()?;
        Ok(Stmt::FnDecl { name: "main".into(), params: vec![], body })
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

    fn skip_type_annotation(&mut self) -> Result<(), String> {
        match self.peek().clone() {
            Token::Ident(_) => { self.advance(); Ok(()) }
            t => Err(format!("Expected type name, got {:?}", t)),
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
            t => Err(format!("Expected field type (int/float), got {:?}", t)),
        }
    }

    fn parse_impl_decl(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'impl'
        let struct_name = self.expect_ident()?;
        self.expect(&Token::LBrace)?;
        let mut methods = Vec::new();
        while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
            methods.push(self.parse_method_decl()?);
        }
        self.expect(&Token::RBrace)?;
        Ok(Stmt::ImplDecl { struct_name, methods })
    }

    fn parse_method_decl(&mut self) -> Result<MethodDecl, String> {
        self.expect(&Token::Fn)?;
        let name = self.expect_ident()?;
        self.expect(&Token::LParen)?;

        let mut has_self = false;
        let mut params   = Vec::new();

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

        self.expect(&Token::RParen)?;
        // optional return-type annotation  : type
        if self.eat(&Token::Colon) { self.skip_type_annotation()?; }
        let body = self.parse_block_stmts()?;
        Ok(MethodDecl { name, params, has_self, body })
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

    /// Returns true when the current token sequence is  (ident | self) DOT ident ASSIGN
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
        // Field assign must be checked before the generic match arms.
        if self.is_field_assign_stmt() {
            return self.parse_field_assign_stmt();
        }

        match self.peek().clone() {
            Token::Let      => self.parse_let(),
            Token::If       => self.parse_if(),
            Token::While    => self.parse_while(),
            Token::For      => self.parse_for(),
            Token::Loop     => self.parse_loop(),
            Token::Return   => self.parse_return(),
            Token::Print    => self.parse_print(),
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
            // compound assignment:  ident += / -= / *= / /= expr ;
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

    fn parse_let(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'let'
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
            t => return Err(format!("Expected compound-assign operator, got {:?}", t)),
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

    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'while'
        self.expect(&Token::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(&Token::RParen)?;
        let body = self.parse_block_stmts()?;
        Ok(Stmt::While { cond, body: Box::new(Stmt::Block(body)) })
    }

    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'for'
        let var = self.expect_ident()?;
        self.expect(&Token::Assign)?;
        let from = self.parse_expr()?;
        if !self.eat(&Token::To) {
            return Err(format!("Expected 'to' in for loop, got {:?}", self.peek().clone()));
        }
        let to = self.parse_expr()?;
        let body = self.parse_block_stmts()?;
        Ok(Stmt::For { var, from, to, body: Box::new(Stmt::Block(body)) })
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

    fn parse_print(&mut self) -> Result<Stmt, String> {
        self.advance(); // 'print'
        let e = self.parse_expr()?;
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
            // negative integer literal:  -N
            Token::Minus => {
                self.advance();
                match self.advance() {
                    Token::Int(n) => MatchPat::Int(-n),
                    t => return Err(format!("Expected integer after '-' in match arm, got {:?}", t)),
                }
            }
            Token::Ident(s) if s == "_" => {
                self.advance();
                MatchPat::Wildcard
            }
            t => return Err(format!("Expected match pattern (integer or _), got {:?}", t)),
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
    //    call_base  :  name(args) / new Name { ... }
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

    /// Parse a function call, struct literal, or primary.
    fn parse_call_base(&mut self) -> Result<Expr, String> {
        // struct literal:  new StructName { field: expr, ... }
        if self.eat(&Token::New) {
            return self.parse_struct_lit();
        }
        // function call:  ident(args)
        if let Token::Ident(name) = self.peek().clone() {
            if matches!(self.peek2(), Token::LParen) {
                self.advance(); // ident
                self.advance(); // (
                let args = self.parse_arg_list()?;
                self.expect(&Token::RParen)?;
                return Ok(Expr::Call { name, args });
            }
        }
        self.parse_primary()
    }

    fn parse_struct_lit(&mut self) -> Result<Expr, String> {
        let name = self.expect_ident()?;
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
            Token::Int(n)    => Ok(Expr::Number(n)),
            Token::Float(f)  => Ok(Expr::Float(f)),
            Token::Str(s)    => Ok(Expr::Str(s)),
            Token::True      => Ok(Expr::Number(1)),
            Token::False     => Ok(Expr::Number(0)),
            Token::Ident(n)  => Ok(Expr::Ident(n)),
            Token::SelfKw    => Ok(Expr::Ident("self".into())),
            Token::LParen    => {
                let e = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(e)
            }
            t => Err(format!("Expected an expression, got {:?}", t)),
        }
    }
}
