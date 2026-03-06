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
    Const,     // const — immutable constant  (from Rust / C++)
    Func,      // func  — function declaration
    Return,
    If,
    Else,
    Unless,    // unless — inverted if         (from Ruby)
    While,
    Do,        // do    — do-while loop
    For,
    In,        // in    — for i in range
    Loop,
    Repeat,    // repeat N { } — repeat loop  (from Lua / Pascal)
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
    // New keywords
    Enum,      // enum   — integer-backed enum      (from Rust / Swift)
    Defer,     // defer  — deferred execution       (from Go)
    Import,    // import — multi-file import
    Extern,    // extern — external C function declaration

    // ── Range operators ───────────────────────────────────────────────────
    DotDot,    // ..    exclusive range
    DotDotEq,  // ..=   inclusive range

    // ── Operators ─────────────────────────────────────────────────────────
    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /
    Percent,     // %
    StarStar,    // **   power operator         (from Python)
    EqEq,        // ==
    BangEq,      // !=
    Lt,          // <
    LtEq,        // <=
    Gt,          // >
    GtEq,        // >=
    Amp,         // &    address-of operator
    AndAnd,      // &&
    OrOr,        // ||
    Bang,        // !
    Assign,      // =
    FatArrow,    // =>
    PipeGt,      // |>   pipe operator          (from Elixir / F#)
    Question,    // ?    ternary / null-coalesce (from C / Kotlin)
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
    LBracket,  // [    array indexing          (from Python / JS)
    RBracket,  // ]
    Semicolon, // ;
    Colon,     // :
    Comma,     // ,
    Dot,       // .

    // ── Identifier ────────────────────────────────────────────────────────
    Ident(String),

    Eof,
}
