#![allow(dead_code)]

pub use crate::lexer::token::InterpolPart;

// ── Binary operators ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Pow,  // **  power operator      (Python)
    Xor,  // ^   bitwise XOR        (C / Java)
    Gt, Lt, Ge, Le, Eq, Ne,
    And, Or,
}

// ── Unary operators ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neg,    // -x
    Not,    // !x
    BitNot, // ~x  bitwise NOT       (C / Java)
}

// ── Access modifier ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Access {
    Public,
    Private,
}

// ── Struct field type ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum FieldType {
    Int,
    Float,
    Named(String),
}

// ── Class field declaration ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FieldDecl {
    pub name:   String,
    pub ty:     FieldType,
    pub access: Access,
}

// ── Match arm pattern ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum MatchPat {
    Int(i64),
    Wildcard,                        // _
    EnumVariant(String, String),     // EnumName.Variant   (Rust / Swift)
}

// ── Match arm (statement form) ────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pat:  MatchPat,
    pub body: Vec<Stmt>,
}

// ── Match arm (expression form) ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MatchArmExpr {
    pub pat: MatchPat,
    pub val: Expr,
}

// ── Method declaration ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MethodDecl {
    pub name:      String,
    pub params:    Vec<(String, Option<Expr>)>, // (name, default_value)
    pub has_self:  bool,
    pub is_static: bool,
    pub body:      Vec<Stmt>,
    pub access:    Access,
}

// ── Function default parameter ────────────────────────────────────────────────

/// A single function parameter: (name, optional_default_expr)
pub type Param = (String, Option<Expr>);

// ── Expressions ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64),
    Float(f64),
    Str(String),
    /// $"Hello {name}!"  — interpolated string   (C# / Kotlin)
    Interpolated(Vec<InterpolPart>),
    Ident(String),
    Binary(Box<Expr>, BinOp, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    /// cond ? then : els — ternary operator       (C / Java)
    Ternary { cond: Box<Expr>, then: Box<Expr>, els: Box<Expr> },
    /// name(args)  — regular function call
    Call { name: String, args: Vec<Expr> },
    /// Type::method(args)  — static method call   (C++ / Rust)
    StaticCall { type_name: String, method: String, args: Vec<Expr> },
    /// obj.field
    FieldAccess { obj: Box<Expr>, field: String },
    /// obj.method(args)
    MethodCall { obj: Box<Expr>, method: String, args: Vec<Expr> },
    /// StructName { field: expr, ... }  — struct literal (no `new`)
    StructLit { name: String, fields: Vec<(String, Expr)> },
    /// new ClassName(args)  — constructor call
    ConstructorCall { class: String, args: Vec<Expr> },
    /// [expr, ...]  — array literal               (Python / JS)
    ArrayLit(Vec<Expr>),
    /// expr[idx]   — array / index access         (Python / JS)
    Index { arr: Box<Expr>, idx: Box<Expr> },
    /// (a, b, ...)  — tuple literal               (Python / Rust)
    Tuple(Vec<Expr>),
    /// |params| expr  — lambda / closure          (Rust / Python)
    Lambda { params: Vec<String>, body: Box<Expr> },
    /// match expr { pat => val, ... }  — match as expression
    MatchExpr { expr: Box<Expr>, arms: Vec<MatchArmExpr> },
    /// readInt()  — reads one i64 from stdin via scanf
    Input,
    /// readFloat()  — reads one f64 from stdin via scanf
    InputFloat,
    /// &expr — address of a variable
    AddrOf(Box<Expr>),
    /// *expr — dereference a pointer
    Deref(Box<Expr>),
    /// cstr("literal") — address of a null-terminated C string global
    CStr(String),
}

// ── Statements ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Stmt {
    /// var name = expr;
    Let { name: String, expr: Expr },
    /// var (a, b) = expr;  — tuple destructuring   (Python / Rust)
    LetTuple { names: Vec<String>, expr: Expr },
    /// const NAME = expr;                           (Rust / C++)
    Const { name: String, expr: Expr },
    /// name = expr;
    Assign { name: String, expr: Expr },
    /// obj.field = expr;
    FieldAssign { obj: Expr, field: String, val: Expr },
    /// arr[idx] = val;                              (Python / JS)
    IndexAssign { arr: Box<Expr>, idx: Box<Expr>, val: Expr },
    /// bare expression statement
    Expr(Expr),
    /// println(expr);
    Print(Expr),
    /// return expr;
    Return(Expr),
    /// { stmts }
    Block(Vec<Stmt>),
    /// if (cond) { then } [else { els }]
    If {
        cond: Expr,
        then: Box<Stmt>,
        els:  Option<Box<Stmt>>,
    },
    /// while (cond) { body }
    While {
        cond: Expr,
        body: Box<Stmt>,
    },
    /// do { body } while (cond);
    DoWhile {
        body: Box<Stmt>,
        cond: Expr,
    },
    /// for var in from..to { body }   — range iteration
    For {
        var:       String,
        from:      Expr,
        to:        Expr,
        inclusive: bool,
        body:      Box<Stmt>,
    },
    /// for x in array { body }        — array iteration  (Python)
    ForIn {
        var:  String,
        iter: Expr,
        body: Box<Stmt>,
    },
    /// loop { body }
    Loop { body: Box<Stmt> },
    /// break;
    Break,
    /// continue;
    Continue,
    /// match expr { pat => { body }, ... }
    Match { expr: Expr, arms: Vec<MatchArm> },
    /// func name(params) { body }
    FnDecl {
        name:   String,
        params: Vec<Param>,
        body:   Vec<Stmt>,
    },
    /// @annotation
    Annotation { name: String },
    /// struct Name { field: type, ... }
    StructDecl {
        name:   String,
        fields: Vec<(String, FieldType)>,
    },
    /// impl Name { pub func method(...) { ... } }
    ImplDecl {
        struct_name: String,
        methods:     Vec<MethodDecl>,
    },
    /// impl Trait for Type { ... }    — trait implementation  (Rust / Swift)
    ImplTrait {
        trait_name: String,
        for_type:   String,
        methods:    Vec<MethodDecl>,
    },
    /// trait Name { func sig(self); ... }  — trait declaration  (Rust / Swift)
    TraitDecl {
        name:     String,
        /// list of (method_name, param_names)
        methods:  Vec<(String, Vec<String>)>,
    },
    /// class Name [extends Parent] { ... }
    ClassDecl {
        name:    String,
        parent:  Option<String>,
        fields:  Vec<FieldDecl>,
        methods: Vec<MethodDecl>,
    },
    /// enum Name { Variant, ... }          (Rust / Swift)
    EnumDecl {
        name:     String,
        variants: Vec<String>,
    },
    /// defer stmt;                          (Go) — executes at function exit
    Defer(Box<Stmt>),
    /// import "module";                     — multi-file import
    Import { path: String },
    /// extern func name(p0, p1, ...): ret;  — declare external C function
    ExternFn {
        name:     String,
        params:   usize,
        variadic: bool,
    },
}
