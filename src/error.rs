/// Unified compile error type for the Orbitron compiler.
#[derive(Debug)]
pub enum CompileError {
    Lex(String),
    Parse(String),
    Codegen(String),
    Io(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Lex(msg)     => write!(f, "Lexer error: {}", msg),
            CompileError::Parse(msg)   => write!(f, "Parse error: {}", msg),
            CompileError::Codegen(msg) => write!(f, "Codegen error: {}", msg),
            CompileError::Io(msg)      => write!(f, "I/O error: {}", msg),
        }
    }
}
