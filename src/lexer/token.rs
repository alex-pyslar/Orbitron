/// Part of a `$"..."` or `"...\{...}"` interpolated string.
#[derive(Debug, Clone, PartialEq)]
pub enum InterpolPart {
    Lit(String),  // literal text segment
    Var(String),  // `{ident}` or `\{ident}` hole — variable name to inline
}

/// All tokens produced by the Orbitron lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // ── Literals ──────────────────────────────────────────────────────────
    Int(i64),
    Float(f64),
    Str(String),
    /// Interpolated string literal — `$"Hello, {name}!"` or `"Hello \{name}!"`
    InterpolStr(Vec<InterpolPart>),

    // ── Keywords ──────────────────────────────────────────────────────────
    Var,       // var   — variable declaration (primary keyword)
    Let,       // let   — immutable binding (kept for backward compat)
    Mut,       // mut   — mutable modifier
    Const,     // const — compile-time constant           (Rust / C++)
    Func,      // func  — function declaration (old syntax, kept for compat)
    Fn,        // fn    — function declaration (Rust-style)
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
    Pub,       // pub   — alias for public (Rust-style)
    Priv,      // priv  — alias for private (short form)
    Prot,      // prot  — alias for protected (short form)
    Public,    // public   — fully visible                (Java / Kotlin)
    Private,   // private  — class-local visibility       (Java / Kotlin)
    Protected, // protected — subclass visibility         (Java / Kotlin)
    Internal,  // internal — module-level visibility      (Kotlin)
    Static,    // static — static method modifier        (Java / C++)
    Trait,     // trait  — trait / interface              (Rust / Swift)
    Extends,   // extends — class inheritance             (Java / Kotlin)
    // Features
    Enum,      // enum   — integer-backed enum           (Rust / Swift)
    Defer,     // defer  — deferred execution            (Go)
    Import,    // import — multi-file import
    Extern,    // extern — external C function declaration
    // New keywords
    Type,      // type  — type alias                    (Swift / Kotlin)
    Where,     // where — constraint placeholder         (Rust / Haskell)
    // Concurrency keywords
    Go,        // go    — goroutine spawn                (Go)
    Async,     // async — async function / block         (Rust / Kotlin)
    Await,     // await — async wait                     (Rust / Kotlin)
    Launch,    // launch — coroutine launch scope        (Kotlin)
    Chan,      // chan  — channel creation               (Go)
    // Annotations
    At,        // @     — decorator/annotation prefix   (Python / Java)
    // Hash directives (kept for backward compat — parsed same as plain keyword)
    #[allow(dead_code)]
    HashImport, // #import  — old-style import directive (deprecated)
    #[allow(dead_code)]
    HashConst,  // #const   — old-style const directive (deprecated)

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
    Bang,        // !    logical NOT / macro call marker
    Assign,      // =
    FatArrow,    // =>   match arm / expression-body function
    Arrow,       // ->   return-type annotation        (Rust / Swift)
    PipeGt,      // |>   pipe operator                 (Elixir / F#)
    Question,    // ?    ternary                       (C / Java)
    QuestionDot, // ?.   optional chaining             (Swift / Kotlin)
    Elvis,       // ?:   null-coalescing / Elvis        (Kotlin / Groovy)
    ColonColon,  // ::   static method / namespace      (C++ / Rust)
    ChanOp,      // <-   channel send / receive        (Go)
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
    Hash,      // #    (standalone, for future use)

    // ── Identifier ────────────────────────────────────────────────────────
    Ident(String),

    Eof,
}
