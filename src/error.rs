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
            CompileError::Lex(msg)     => write!(f, "Ошибка лексера: {}", msg),
            CompileError::Parse(msg)   => write!(f, "Ошибка парсера: {}", msg),
            CompileError::Codegen(msg) => write!(f, "Ошибка кодогенерации: {}", msg),
            CompileError::Io(msg)      => write!(f, "Ошибка ввода/вывода: {}", msg),
        }
    }
}
