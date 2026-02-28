// ── Binary operators ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Gt, Lt, Ge, Le, Eq, Ne,
    And, Or,
}

// ── Unary operators ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neg, // -x
    Not, // !x
}

// ── Struct field type ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum FieldType {
    Int,
    Float,
    Named(String), // for future nested structs
}

// ── Match arm pattern ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum MatchPat {
    Int(i64),
    Wildcard, // _
}

// ── Match arm ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pat:  MatchPat,
    pub body: Vec<Stmt>,
}

// ── Method declaration (inside impl block) ───────────────────────────────────

#[derive(Debug, Clone)]
pub struct MethodDecl {
    pub name:     String,
    pub params:   Vec<String>, // excludes `self`
    pub has_self: bool,
    pub body:     Vec<Stmt>,
}

// ── Expressions ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64),
    Float(f64),
    Str(String),
    Ident(String),
    Binary(Box<Expr>, BinOp, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Call { name: String, args: Vec<Expr> },
    /// obj.field
    FieldAccess { obj: Box<Expr>, field: String },
    /// obj.method(args)
    MethodCall { obj: Box<Expr>, method: String, args: Vec<Expr> },
    /// new StructName { field: expr, ... }
    StructLit { name: String, fields: Vec<(String, Expr)> },
}

// ── Statements ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Stmt {
    /// let name = expr;
    Let { name: String, expr: Expr },
    /// name = expr;
    Assign { name: String, expr: Expr },
    /// obj.field = expr;
    FieldAssign { obj: Expr, field: String, val: Expr },
    /// bare expression statement
    Expr(Expr),
    /// print expr;
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
    /// for var = from to to { body }
    For {
        var:  String,
        from: Expr,
        to:   Expr,
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
    /// fn name(params) { body }  /  main { body }
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
    /// impl Name { fn method(...) { ... } }
    ImplDecl {
        struct_name: String,
        methods:     Vec<MethodDecl>,
    },
}
