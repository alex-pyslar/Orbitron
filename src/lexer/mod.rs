pub mod token;
pub use token::{InterpolPart, Token};

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
                    None       => return Err("Unterminated escape in string".into()),
                },
                Some(c) => s.push(c),
                None    => return Err(format!("Unterminated string at line {}", self.line)),
            }
        }
    }

    /// Lex a `$"..."` interpolated string.
    /// Supports `{ident}` holes for variable interpolation  (C# / Kotlin).
    fn read_interp_string(&mut self) -> Result<Token, String> {
        self.advance(); // consume '$'
        if self.peek() != Some('"') {
            return Err(format!("Expected '\"' after '$' at line {}", self.line));
        }
        self.advance(); // consume '"'

        let mut parts: Vec<InterpolPart> = Vec::new();
        let mut lit = String::new();

        loop {
            match self.advance() {
                Some('"') => {
                    if !lit.is_empty() { parts.push(InterpolPart::Lit(std::mem::take(&mut lit))); }
                    return Ok(Token::InterpolStr(parts));
                }
                Some('{') => {
                    if !lit.is_empty() { parts.push(InterpolPart::Lit(std::mem::take(&mut lit))); }
                    // Read identifier until '}'
                    let mut ident = String::new();
                    loop {
                        match self.advance() {
                            Some('}') => break,
                            Some(c) if c.is_alphanumeric() || c == '_' => ident.push(c),
                            Some(c) => return Err(format!(
                                "Unexpected character '{}' in string interpolation (line {})",
                                c, self.line
                            )),
                            None => return Err("Unterminated string interpolation".into()),
                        }
                    }
                    if ident.is_empty() {
                        return Err("Empty interpolation '{}' in string".into());
                    }
                    parts.push(InterpolPart::Var(ident));
                }
                Some('\\') => match self.advance() {
                    Some('n')  => lit.push('\n'),
                    Some('t')  => lit.push('\t'),
                    Some('\\') => lit.push('\\'),
                    Some('"')  => lit.push('"'),
                    Some('{')  => lit.push('{'),
                    Some(c)    => { lit.push('\\'); lit.push(c); }
                    None       => return Err("Unterminated escape in interpolated string".into()),
                },
                Some(c) => lit.push(c),
                None    => return Err(format!("Unterminated interpolated string at line {}", self.line)),
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
            "const"    => Token::Const,     // Rust/C++
            "func"     => Token::Func,
            "return"   => Token::Return,
            "if"       => Token::If,
            "else"     => Token::Else,
            "unless"   => Token::Unless,    // Ruby
            "while"    => Token::While,
            "do"       => Token::Do,
            "for"      => Token::For,
            "in"       => Token::In,
            "loop"     => Token::Loop,
            "repeat"   => Token::Repeat,    // Lua/Pascal
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
            "static"   => Token::Static,    // Java/C++
            "trait"    => Token::Trait,     // Rust/Swift
            "extends"  => Token::Extends,   // Java/Kotlin
            "enum"     => Token::Enum,      // Rust/Swift
            "defer"    => Token::Defer,     // Go
            "import"   => Token::Import,    // multi-file import
            "extern"   => Token::Extern,    // external C declaration
            _          => Token::Ident(s),
        }
    }

    pub fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace_and_comments();
        match self.peek() {
            None    => Ok(Token::Eof),
            Some(c) => match c {
                // Interpolated string: $"Hello {name}!"  (C# / Kotlin)
                '$'                          => self.read_interp_string(),
                '"'                          => self.read_string(),
                '0'..='9'                    => Ok(self.read_number()),
                'a'..='z' | 'A'..='Z' | '_' => Ok(self.read_ident()),
                // Annotation: @name
                '@' => { self.advance(); Ok(Token::At) }
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
                    else if self.peek() == Some('>') { self.advance(); Ok(Token::Arrow) }
                    else { Ok(Token::Minus) }
                }
                // ** power (Python), *= compound, * multiply
                '*' => {
                    self.advance();
                    if self.peek() == Some('*') { self.advance(); Ok(Token::StarStar) }
                    else if self.peek() == Some('=') { self.advance(); Ok(Token::StarAssign) }
                    else { Ok(Token::Star) }
                }
                '/' => {
                    self.advance();
                    if self.peek() == Some('=') { self.advance(); Ok(Token::SlashAssign) }
                    else { Ok(Token::Slash) }
                }
                '%' => {
                    self.advance();
                    if self.peek() == Some('=') { self.advance(); Ok(Token::PercentAssign) }
                    else { Ok(Token::Percent) }
                }
                // ^ XOR (C / Java), ^= compound
                '^' => {
                    self.advance();
                    if self.peek() == Some('=') { self.advance(); Ok(Token::CaretAssign) }
                    else { Ok(Token::Caret) }
                }
                // ~ bitwise NOT (C / Java)
                '~' => { self.advance(); Ok(Token::Tilde) }
                '(' => { self.advance(); Ok(Token::LParen) }
                ')' => { self.advance(); Ok(Token::RParen) }
                '{' => { self.advance(); Ok(Token::LBrace) }
                '}' => { self.advance(); Ok(Token::RBrace) }
                // Array brackets (Python / JS)
                '[' => { self.advance(); Ok(Token::LBracket) }
                ']' => { self.advance(); Ok(Token::RBracket) }
                ';' => { self.advance(); Ok(Token::Semicolon) }
                // :: static method / namespace (C++ / Rust), : type annotation
                ':' => {
                    self.advance();
                    if self.peek() == Some(':') { self.advance(); Ok(Token::ColonColon) }
                    else { Ok(Token::Colon) }
                }
                ',' => { self.advance(); Ok(Token::Comma) }
                // Ternary ? (C / Kotlin)
                '?' => { self.advance(); Ok(Token::Question) }
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
                    else { Ok(Token::Amp) }
                }
                // |> pipe (Elixir / F#), || logical-or, | lambda/bitwise-or
                '|' => {
                    self.advance();
                    if self.peek() == Some('|') { self.advance(); Ok(Token::OrOr) }
                    else if self.peek() == Some('>') { self.advance(); Ok(Token::PipeGt) }
                    else { Ok(Token::Pipe) }
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
