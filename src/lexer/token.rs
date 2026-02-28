/// All tokens produced by the Orbitron lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // ── Literals ──────────────────────────────────────────────────────────
    Int(i64),
    Float(f64),
    Str(String),

    // ── Keywords ──────────────────────────────────────────────────────────
    Var,      // var   — variable declaration
    Func,     // func  — function declaration
    Return,
    If,
    Else,
    While,
    Do,       // do    — do-while loop
    For,
    In,       // in    — for i in range
    Loop,
    Match,
    Println,  // println(expr) — print with newline
    True,
    False,
    Break,
    Continue,
    // OOP
    Struct,
    Impl,
    Class,
    SelfKw,   // self
    New,      // new   — constructor call
    Init,     // init  — class constructor block
    Pub,
    Private,

    // ── Range operators ───────────────────────────────────────────────────
    DotDot,   // ..    exclusive range
    DotDotEq, // ..=   inclusive range

    // ── Operators ─────────────────────────────────────────────────────────
    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /
    Percent,     // %
    EqEq,        // ==
    BangEq,      // !=
    Lt,          // <
    LtEq,        // <=
    Gt,          // >
    GtEq,        // >=
    AndAnd,      // &&
    OrOr,        // ||
    Bang,        // !
    Assign,      // =
    FatArrow,    // =>
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
