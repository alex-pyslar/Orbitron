/// All tokens produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // ── Literals ──────────────────────────────────────────────────────────
    Int(i64),
    Float(f64),
    Str(String),

    // ── Keywords ──────────────────────────────────────────────────────────
    Let,
    Fn,
    Return,
    If,
    Else,
    While,
    For,
    To,
    Print,
    Main,
    True,
    False,
    Break,
    Continue,
    // OOP
    Struct,
    Impl,
    SelfKw,
    New,
    // Control flow
    Loop,
    Match,

    // ── Operators ─────────────────────────────────────────────────────────
    Plus,      // +
    Minus,     // -
    Star,      // *
    Slash,     // /
    Percent,   // %
    EqEq,      // ==
    BangEq,    // !=
    Lt,        // <
    LtEq,      // <=
    Gt,        // >
    GtEq,      // >=
    AndAnd,    // &&
    OrOr,      // ||
    Bang,      // !
    Assign,    // =
    FatArrow,  // =>
    // Compound assignment
    PlusAssign,  // +=
    MinusAssign, // -=
    StarAssign,  // *=
    SlashAssign, // /=

    // ── Punctuation ───────────────────────────────────────────────────────
    LParen,    // (
    RParen,    // )
    LBrace,    // {
    RBrace,    // }
    Semicolon, // ;
    Colon,     // :
    Comma,     // ,
    Dot,       // .

    // ── Identifier ────────────────────────────────────────────────────────
    Ident(String),

    Eof,
}

// ── Lexer ──────────────────────────────────────────────────────────────────

pub struct Lexer {
    input: Vec<char>,
    pos:   usize,
    pub line: usize,
    pub col:  usize,
}

impl Lexer {
    pub fn new(src: &str) -> Self {
        Lexer { input: src.chars().collect(), pos: 0, line: 1, col: 1 }
    }

    fn peek(&self) -> Option<char>  { self.input.get(self.pos).copied() }
    fn peek2(&self) -> Option<char> { self.input.get(self.pos + 1).copied() }

    fn advance(&mut self) -> Option<char> {
        let ch = self.input.get(self.pos).copied()?;
        self.pos += 1;
        if ch == '\n' { self.line += 1; self.col = 1; } else { self.col += 1; }
        Some(ch)
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // whitespace
            while self.peek().map_or(false, |c| c.is_whitespace()) {
                self.advance();
            }
            // line comment  //
            if self.peek() == Some('/') && self.peek2() == Some('/') {
                while self.peek().map_or(false, |c| c != '\n') { self.advance(); }
                continue;
            }
            // block comment  /* … */
            if self.peek() == Some('/') && self.peek2() == Some('*') {
                self.advance(); self.advance(); // consume /*
                loop {
                    match self.advance() {
                        Some('*') if self.peek() == Some('/') => { self.advance(); break; }
                        None => break,
                        _ => {}
                    }
                }
                continue;
            }
            break;
        }
    }

    fn read_string(&mut self) -> Result<Token, String> {
        self.advance(); // opening "
        let mut s = String::new();
        loop {
            match self.advance() {
                Some('"')  => return Ok(Token::Str(s)),
                Some('\\') => match self.advance() {
                    Some('n')  => s.push('\n'),
                    Some('t')  => s.push('\t'),
                    Some('\\') => s.push('\\'),
                    Some('"')  => s.push('"'),
                    Some(c)    => { s.push('\\'); s.push(c); }
                    None       => return Err("Unterminated escape in string".into()),
                },
                Some(c) => s.push(c),
                None    => return Err(format!("Unterminated string at line {}", self.line)),
            }
        }
    }

    fn read_number(&mut self) -> Token {
        let mut s = String::new();
        while self.peek().map_or(false, |c| c.is_ascii_digit()) {
            s.push(self.advance().unwrap());
        }
        // optional decimal part
        if self.peek() == Some('.')
            && self.peek2().map_or(false, |c| c.is_ascii_digit())
        {
            s.push(self.advance().unwrap()); // '.'
            while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                s.push(self.advance().unwrap());
            }
            return Token::Float(s.parse().unwrap());
        }
        Token::Int(s.parse().unwrap())
    }

    fn read_ident(&mut self) -> Token {
        let mut s = String::new();
        while self.peek().map_or(false, |c| c.is_alphanumeric() || c == '_') {
            s.push(self.advance().unwrap());
        }
        match s.as_str() {
            "let"      => Token::Let,
            "fn"       => Token::Fn,
            "return"   => Token::Return,
            "if"       => Token::If,
            "else"     => Token::Else,
            "while"    => Token::While,
            "for"      => Token::For,
            "to"       => Token::To,
            "print"    => Token::Print,
            "main"     => Token::Main,
            "true"     => Token::True,
            "false"    => Token::False,
            "break"    => Token::Break,
            "continue" => Token::Continue,
            "struct"   => Token::Struct,
            "impl"     => Token::Impl,
            "self"     => Token::SelfKw,
            "new"      => Token::New,
            "loop"     => Token::Loop,
            "match"    => Token::Match,
            _          => Token::Ident(s),
        }
    }

    pub fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace_and_comments();
        match self.peek() {
            None    => Ok(Token::Eof),
            Some(c) => match c {
                '"'                          => self.read_string(),
                '0'..='9'                    => Ok(self.read_number()),
                'a'..='z' | 'A'..='Z' | '_' => Ok(self.read_ident()),
                '.' => { self.advance(); Ok(Token::Dot) }
                '+' => {
                    self.advance();
                    if self.peek() == Some('=') { self.advance(); Ok(Token::PlusAssign) }
                    else { Ok(Token::Plus) }
                }
                '-' => {
                    self.advance();
                    if self.peek() == Some('=') { self.advance(); Ok(Token::MinusAssign) }
                    else { Ok(Token::Minus) }
                }
                '*' => {
                    self.advance();
                    if self.peek() == Some('=') { self.advance(); Ok(Token::StarAssign) }
                    else { Ok(Token::Star) }
                }
                '/' => {
                    self.advance();
                    if self.peek() == Some('=') { self.advance(); Ok(Token::SlashAssign) }
                    else { Ok(Token::Slash) }
                }
                '%' => { self.advance(); Ok(Token::Percent) }
                '(' => { self.advance(); Ok(Token::LParen) }
                ')' => { self.advance(); Ok(Token::RParen) }
                '{' => { self.advance(); Ok(Token::LBrace) }
                '}' => { self.advance(); Ok(Token::RBrace) }
                ';' => { self.advance(); Ok(Token::Semicolon) }
                ':' => { self.advance(); Ok(Token::Colon) }
                ',' => { self.advance(); Ok(Token::Comma) }
                '=' => {
                    self.advance();
                    if self.peek() == Some('=') { self.advance(); Ok(Token::EqEq) }
                    else if self.peek() == Some('>') { self.advance(); Ok(Token::FatArrow) }
                    else { Ok(Token::Assign) }
                }
                '!' => {
                    self.advance();
                    if self.peek() == Some('=') { self.advance(); Ok(Token::BangEq) }
                    else { Ok(Token::Bang) }
                }
                '<' => {
                    self.advance();
                    if self.peek() == Some('=') { self.advance(); Ok(Token::LtEq) }
                    else { Ok(Token::Lt) }
                }
                '>' => {
                    self.advance();
                    if self.peek() == Some('=') { self.advance(); Ok(Token::GtEq) }
                    else { Ok(Token::Gt) }
                }
                '&' => {
                    self.advance();
                    if self.peek() == Some('&') { self.advance(); Ok(Token::AndAnd) }
                    else { Err(format!("Single '&' is not valid (line {})", self.line)) }
                }
                '|' => {
                    self.advance();
                    if self.peek() == Some('|') { self.advance(); Ok(Token::OrOr) }
                    else { Err(format!("Single '|' is not valid (line {})", self.line)) }
                }
                other => Err(format!("Unexpected character '{}' at line {}:{}", other, self.line, self.col)),
            }
        }
    }

    /// Tokenise the entire source and return the token list.
    pub fn tokenize(src: &str) -> Result<Vec<Token>, String> {
        let mut lex = Lexer::new(src);
        let mut out = Vec::new();
        loop {
            let tok = lex.next_token()?;
            let done = tok == Token::Eof;
            out.push(tok);
            if done { break; }
        }
        Ok(out)
    }
}
