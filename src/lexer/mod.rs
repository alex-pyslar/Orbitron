pub mod token;
pub use token::Token;

// ── Lexer ───────────────────────────────────────────────────────────────────

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
    fn peek3(&self) -> Option<char> { self.input.get(self.pos + 2).copied() }

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
                    None       => return Err("Незакрытый escape в строке".into()),
                },
                Some(c) => s.push(c),
                None    => return Err(format!("Незакрытая строка на строке {}", self.line)),
            }
        }
    }

    fn read_number(&mut self) -> Token {
        let mut s = String::new();
        while self.peek().map_or(false, |c| c.is_ascii_digit()) {
            s.push(self.advance().unwrap());
        }
        // optional decimal part — only if digit follows the dot
        if self.peek() == Some('.')
            && self.peek2().map_or(false, |c| c.is_ascii_digit())
            && self.peek3().map_or(true, |c| c != '.') // avoid `1..` being parsed as 1. followed by .
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
            "var"      => Token::Var,
            "func"     => Token::Func,
            "return"   => Token::Return,
            "if"       => Token::If,
            "else"     => Token::Else,
            "while"    => Token::While,
            "do"       => Token::Do,
            "for"      => Token::For,
            "in"       => Token::In,
            "loop"     => Token::Loop,
            "match"    => Token::Match,
            "println"  => Token::Println,
            "true"     => Token::True,
            "false"    => Token::False,
            "break"    => Token::Break,
            "continue" => Token::Continue,
            "struct"   => Token::Struct,
            "impl"     => Token::Impl,
            "class"    => Token::Class,
            "self"     => Token::SelfKw,
            "new"      => Token::New,
            "init"     => Token::Init,
            "pub"      => Token::Pub,
            "private"  => Token::Private,
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
                // Range operators: ..= and ..
                '.' => {
                    self.advance();
                    if self.peek() == Some('.') {
                        self.advance();
                        if self.peek() == Some('=') {
                            self.advance();
                            Ok(Token::DotDotEq)
                        } else {
                            Ok(Token::DotDot)
                        }
                    } else {
                        Ok(Token::Dot)
                    }
                }
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
                    else { Err(format!("Одиночный '&' недопустим (строка {})", self.line)) }
                }
                '|' => {
                    self.advance();
                    if self.peek() == Some('|') { self.advance(); Ok(Token::OrOr) }
                    else { Err(format!("Одиночный '|' недопустим (строка {})", self.line)) }
                }
                other => Err(format!("Неожиданный символ '{}' на строке {}:{}", other, self.line, self.col)),
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
