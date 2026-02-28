#![allow(dead_code)]

// ── Binary operators ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Gt, Lt, Ge, Le, Eq, Ne,
    And, Or,
}

// ── Unary operators ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neg, // -x
    Not, // !x
}

// ── Access modifier ───────────────────────────────────────────────────────────

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
    Wildcard, // _
}

// ── Match arm ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pat:  MatchPat,
    pub body: Vec<Stmt>,
}

// ── Method declaration ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MethodDecl {
    pub name:     String,
    pub params:   Vec<String>, // excludes `self`
    pub has_self: bool,
    pub body:     Vec<Stmt>,
    pub access:   Access,
}

// ── Expressions ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64),
    Float(f64),
    Str(String),
    Ident(String),
    Binary(Box<Expr>, BinOp, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    /// name(args)  — regular function call
    Call { name: String, args: Vec<Expr> },
    /// obj.field
    FieldAccess { obj: Box<Expr>, field: String },
    /// obj.method(args)
    MethodCall { obj: Box<Expr>, method: String, args: Vec<Expr> },
    /// StructName { field: expr, ... }  — struct literal (no `new`)
    StructLit { name: String, fields: Vec<(String, Expr)> },
    /// new ClassName(args)  — constructor call
    ConstructorCall { class: String, args: Vec<Expr> },
    /// readInt()  — reads one i64 from stdin via scanf
    Input,
    /// readFloat()  — reads one f64 from stdin via scanf
    InputFloat,
}

// ── Statements ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Stmt {
    /// var name = expr;
    Let { name: String, expr: Expr },
    /// name = expr;
    Assign { name: String, expr: Expr },
    /// obj.field = expr;
    FieldAssign { obj: Expr, field: String, val: Expr },
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
    /// for var in from..to { body }  (inclusive=false → exclusive range)
    /// for var in from..=to { body } (inclusive=true  → inclusive range)
    /// Multi-range desugars to nested For at parse time.
    For {
        var:       String,
        from:      Expr,
        to:        Expr,
        inclusive: bool,
        body:      Box<Stmt>,
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
        params: Vec<String>,
        body:   Vec<Stmt>,
    },
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
    /// class Name { [pub|private] field: type, ... init(...) { } pub func method(...) { } }
    ClassDecl {
        name:    String,
        fields:  Vec<FieldDecl>,
        methods: Vec<MethodDecl>,
    },
}
