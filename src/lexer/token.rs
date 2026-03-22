/// Part of a `$"..."` interpolated string.
#[derive(Debug, Clone, PartialEq)]
pub enum InterpolPart {
    Lit(String),  // literal text segment
    Var(String),  // `{ident}` hole — variable name to inline
}

/// All tokens produced by the Orbitron lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // ── Literals ──────────────────────────────────────────────────────────
    Int(i64),
    Float(f64),
    Str(String),
    /// $"Hello, {name}!" — interpolated string literal
    InterpolStr(Vec<InterpolPart>),

    // ── Keywords ──────────────────────────────────────────────────────────
    Var,       // var   — variable declaration
    Const,     // const — immutable constant           (Rust / C++)
    Func,      // func  — function declaration
    Return,
    If,
    Else,
    Unless,    // unless — inverted if                  (Ruby)
    While,
    Do,        // do    — do-while loop
    For,
    In,        // in    — for i in range
    Loop,
    Repeat,    // repeat N { } — repeat loop           (Lua / Pascal)
    Match,
    Println,   // println(expr) — print with newline
    True,
    False,
    Break,
    Continue,
    // OOP
    Struct,
    Impl,
    Class,
    SelfKw,    // self
    New,       // new   — constructor call
    Init,      // init  — class constructor block
    Pub,
    Private,
    Static,    // static — static method modifier      (Java / C++)
    Trait,     // trait  — trait / interface            (Rust / Swift)
    Extends,   // extends — class inheritance           (Java / Kotlin)
    // Features
    Enum,      // enum   — integer-backed enum         (Rust / Swift)
    Defer,     // defer  — deferred execution          (Go)
    Import,    // import — multi-file import
    Extern,    // extern — external C function declaration
    // Annotations
    At,        // @     — decorator/annotation prefix  (Python / Java)

    // ── Range operators ───────────────────────────────────────────────────
    DotDot,    // ..    exclusive range
    DotDotEq,  // ..=   inclusive range

    // ── Operators ─────────────────────────────────────────────────────────
    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /
    Percent,     // %
    StarStar,    // **   power operator                (Python)
    EqEq,        // ==
    BangEq,      // !=
    Lt,          // <
    LtEq,        // <=
    Gt,          // >
    GtEq,        // >=
    Amp,         // &    address-of / bitwise-and
    AndAnd,      // &&
    Pipe,        // |    lambda param list / bitwise-or
    OrOr,        // ||
    Caret,       // ^    XOR operator                  (C / Java)
    Tilde,       // ~    bitwise NOT                   (C / Java)
    Bang,        // !
    Assign,      // =
    FatArrow,    // =>
    Arrow,       // ->   return-type annotation        (Rust / Swift)
    PipeGt,      // |>   pipe operator                 (Elixir / F#)
    Question,    // ?    ternary / null-coalesce        (C / Kotlin)
    ColonColon,  // ::   static method / namespace      (C++ / Rust)
    // Compound assignment
    PlusAssign,    // +=
    MinusAssign,   // -=
    StarAssign,    // *=
    SlashAssign,   // /=
    PercentAssign, // %=
    CaretAssign,   // ^=

    // ── Punctuation ───────────────────────────────────────────────────────
    LParen,    // (
    RParen,    // )
    LBrace,    // {
    RBrace,    // }
    LBracket,  // [    array indexing                  (Python / JS)
    RBracket,  // ]
    Semicolon, // ;
    Colon,     // :
    Comma,     // ,
    Dot,       // .

    // ── Identifier ────────────────────────────────────────────────────────
    Ident(String),

    Eof,
}
