# Orbitron Compiler Architecture

Orbitron compiles `.ot` source code to either a native binary (via LLVM IR)
or JVM bytecode (via Java source → javac → `.jar`).

## Compilation Pipeline

### LLVM Backend

```
.ot source file(s)
      │
      ▼  Import Resolver (src/resolver.rs)
   Merged AST (all files + stdlib)
      │
      ▼  Code Generator (src/codegen/)
   LLVM IR (.ll)
      │
      ▼  llc
   Assembly (.s)
      │
      ▼  clang -lm
   Native Binary
```

### JVM Backend

```
.ot source file(s)
      │
      ▼  Import Resolver (src/resolver.rs)
   Merged AST (all files + stdlib)
      │
      ▼  JVM Code Generator (src/jvm/mod.rs)
   Main.java
      │
      ▼  javac
   .class files
      │
      ▼  jar cfm
   <output>.jar
```

---

## Source Layout

```
src/
├── main.rs          — CLI dispatcher: new / build / run / <file.ot>
├── cli.rs           — Backend, BuildOpts, print_help(), parse_build_opts()
├── pipeline.rs      — compile_llvm(), compile_jvm(), find_stdlib(), find_project_root()
├── error.rs         — CompileError type
├── project.rs       — ProjectManifest + load_manifest()
├── resolver.rs      — recursive import resolver (AST merger)
├── lexer/
│   ├── mod.rs       — struct Lexer, tokenize()
│   └── token.rs     — enum Token, keyword table
├── parser/
│   ├── mod.rs       — struct Parser, parse_*() methods
│   └── ast.rs       — AST nodes: Expr, Stmt, BinOp, UnaryOp, ...
├── codegen/
│   ├── mod.rs       — struct CodeGen, generate_program(), save_and_compile()
│   ├── expr.rs      — gen_expr(), gen_binop(), type coercions
│   └── stmt.rs      — gen_stmt() for all statement kinds
└── jvm/
    └── mod.rs       — JvmCodeGen, generate_and_compile()

stdlib/
├── math.ot          — math functions (import "std/math")
├── bits.ot          — bitwise operations (import "std/bits")
└── algo.ot          — utility algorithms (import "std/algo")
```

---

## Entry Point — `src/main.rs`

CLI dispatcher: parses the first argument and delegates to a command handler.

```
orbitron new <name>       → cmd_new()
orbitron build [opts]     → cmd_build_or_run(run=false)
orbitron run   [opts]     → cmd_build_or_run(run=true)
orbitron <file.ot> [opts] → cmd_file()
```

`cmd_build_or_run` searches for `orbitron.toml` up the directory tree from CWD.

---

## CLI — `src/cli.rs`

Everything related to command-line argument parsing:

```rust
pub enum Backend { Llvm, Jvm }

pub struct BuildOpts {
    pub output:     Option<String>,
    pub backend:    Option<Backend>, // CLI override
    pub emit_llvm:  bool,
    pub emit_java:  bool,
    pub save_temps: bool,
    pub verbose:    bool,
}

pub fn print_help()
pub fn parse_build_opts(args: &[String]) -> Result<BuildOpts, String>
```

---

## Pipelines — `src/pipeline.rs`

```rust
// Find stdlib ($ORBITRON_HOME/stdlib/ or {exe_dir}/stdlib/)
pub fn find_stdlib() -> Option<PathBuf>

// Search for orbitron.toml up the directory tree
pub fn find_project_root(start: &Path) -> Option<PathBuf>

// Full LLVM pipeline: resolver → codegen → llc → clang
pub fn compile_llvm(entry, src_root, output, opts) -> Result<(), CompileError>

// Full JVM pipeline: resolver → jvm codegen → javac → jar
pub fn compile_jvm(entry, src_root, output, opts) -> Result<(), String>
```

---

## Project System — `src/project.rs`

Reads and deserializes `orbitron.toml`:

```rust
pub struct ProjectManifest {
    pub project: ProjectSection,   // name, version
    pub build:   BuildSection,     // main, output, backend
}

pub fn load_manifest(root: &Path) -> Result<ProjectManifest, String>
```

Example `orbitron.toml`:

```toml
[project]
name    = "myapp"
version = "0.1.0"

[build]
main    = "src/main.ot"
output  = "bin/myapp"
backend = "llvm"    # or "jvm"
```

---

## Import Resolver — `src/resolver.rs`

```rust
pub fn resolve(
    entry:       &Path,
    src_root:    &Path,
    stdlib_root: Option<&Path>,
    visited:     &mut HashSet<PathBuf>,
) -> Result<Vec<Stmt>, String>
```

**Algorithm:**

1. Canonicalize the path (`fs::canonicalize`) for reliable deduplication.
2. If the path is already in `visited` → return an empty list.
3. Insert the path into `visited` *before* recursing — detects cycles.
4. Lex and parse the file.
5. Walk the AST: `Import` → recurse; everything else → append to result.
6. Return the merged `Vec<Stmt>` (dependencies come before the importing code).

**Path resolution rules:**

```
import "math"       →  {src_root}/math.ot
import "net/http"   →  {src_root}/net/http.ot
import "std/math"   →  {stdlib_root}/math.ot
import "std/bits"   →  {stdlib_root}/bits.ot
import "std/algo"   →  {stdlib_root}/algo.ot
```

**stdlib discovery** (in `src/pipeline.rs::find_stdlib()`):
1. `$ORBITRON_HOME/stdlib/`
2. `{exe_dir}/stdlib/` — folder next to the binary

---

## Lexer — `src/lexer/`

### `token.rs` — the `Token` enum

```rust
pub enum Token {
    // Literals
    Int(i64), Float(f64), Str(String),
    InterpolStr(Vec<InterpolPart>),   // $"...{var}..."

    // Keywords
    Func, Var, Const, Return, If, Else, Unless,
    While, Do, For, In, Loop, Break, Continue,
    Repeat, Match, Struct, Impl, Class, Init, New,
    Pub, Private, SelfKw, True, False,
    Enum, Defer, Import,

    // Identifiers
    Ident(String), Println,

    // Operators
    Plus, Minus, Star, Slash, Percent, StarStar,
    PlusAssign, MinusAssign, StarAssign, SlashAssign,
    Assign, EqEq, BangEq, Lt, LtEq, Gt, GtEq,
    AndAnd, OrOr, Bang, PipeGt,

    // Punctuation
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Semicolon, Colon, Comma, Dot, FatArrow, DotDot, DotDotEq,

    Eof,
}
```

**Lexer highlights:**
- `1..5` is never confused with `1.5`: after `.`, the lexer checks that the next character is also `.`
- `$"text {var} end"` → `Token::InterpolStr(parts)`
- `//` (line) and `/* */` (block) comments are skipped
- `CRLF` is normalized to `LF` before lexing

---

## Parser — `src/parser/`

Recursive-descent parser.

### `ast.rs` — AST nodes

**Expressions (`Expr`):**
```
Number(i64)  |  Float(f64)  |  Str(String)  |  Ident(String)
Binary(Box<Expr>, BinOp, Box<Expr>)
Unary(UnaryOp, Box<Expr>)
Call { name: String, args: Vec<Expr> }
MethodCall { obj: Box<Expr>, method: String, args: Vec<Expr> }
FieldAccess { obj: Box<Expr>, field: String }
StructLit { name: String, fields: Vec<(String, Expr)> }
ConstructorCall { class: String, args: Vec<Expr> }
ArrayLit(Vec<Expr>)
Index { arr: Box<Expr>, idx: Box<Expr> }
Input | InputFloat
AddrOf(Box<Expr>) | Deref(Box<Expr>) | CStr(String)
Ternary { cond, then, els }
Interpolated(Vec<InterpolPart>)
```

**Statements (`Stmt`):**
```
Import { path: String }           ← resolved before codegen, ignored by codegen
FnDecl { name, params, body }
StructDecl { name, fields }
ImplDecl { struct_name, methods }
ClassDecl { name, fields, methods }
EnumDecl { name, variants }
ExternFn { name, params, variadic }
Const { name, expr }
Let { name, expr }                ← var name = expr
Assign { name, expr }
FieldAssign { obj, field, val }
IndexAssign { arr, idx, val }
Return(Expr) | Print(Expr)
If { cond, then, els }
While { cond, body }
DoWhile { body, cond }
For { var, from, to, inclusive, body }
Loop { body }
Break | Continue
Match { expr, arms }
Defer(Box<Stmt>)
Block(Vec<Stmt>) | Expr(Expr)
```

### `mod.rs` — precedence hierarchy (low → high)

```
parse_pipe      |>
parse_ternary   ? :
parse_or        ||
parse_and       &&
parse_cmp       == != < <= > >=
parse_add       + -
parse_mul       * / %
parse_unary     - !  &  *
parse_power     **      (right-associative)
parse_postfix   .field  .method(args)  [idx]
parse_call_base new Name(args) | Name(args) | Name{...} | [arr] | readInt/Float | primary
parse_primary   literal | ident | self | (expr)
```

**Parser edge cases:**

- `looks_like_struct_lit()` — distinguishes `Name { field: val }` from `match expr { ... }`
- `for i in 0..3, j in 0..5` — multi-range for → two nested `Stmt::For` nodes
- `init(params)` → `MethodDecl { name: "new", has_self: true }`
- `i += 1` → desugared to `Stmt::Assign { name: "i", expr: Binary(Ident("i"), Add, 1) }`
- `|>` — desugared to a function call in the parser

---

## LLVM Code Generator — `src/codegen/`

Uses the `inkwell` crate — safe Rust bindings to the LLVM C API.

### `mod.rs` — `CodeGen` struct

```rust
pub struct CodeGen<'ctx> {
    ctx:           &'ctx Context,
    builder:       Builder<'ctx>,
    module:        Module<'ctx>,

    vars:          HashMap<String, Var<'ctx>>,   // name → (ptr, kind)
    i64_ty:        IntType<'ctx>,
    f64_ty:        FloatType<'ctx>,

    struct_types:  HashMap<String, StructType<'ctx>>,
    struct_fields: HashMap<String, Vec<(String, bool)>>,

    loop_stack:    Vec<(BasicBlock<'ctx>, BasicBlock<'ctx>)>,

    consts:   HashMap<String, ConstVal>,
    enums:    HashMap<String, HashMap<String, i64>>,
    deferred: Vec<Stmt>,
}
```

### Three-pass Generation (`generate_program`)

| Pass  | What it does |
|-------|--------------|
| **0** | Declare LLVM types for struct/class; collect enum variants; register constants |
| **1** | Forward-declare signatures for all functions and methods |
| **2** | Generate function and method bodies |

### Method Naming in LLVM IR

```
Orbitron:  pub func tick(self)      // in class/impl Counter
LLVM:      define i64 @Counter_tick(ptr %0)

Orbitron:  init(v: int, s: int)
LLVM:      define i64 @Counter_new(ptr %0, i64 %1, i64 %2)
```

### Feature Implementation

| Feature       | Implementation |
|---------------|----------------|
| `const`       | `HashMap<String, ConstVal>` — checked before var lookup |
| `**`          | Calls libm `pow()`; result cast to i64 if both operands are int |
| `\|>`         | Desugared in the parser → plain `Call` node |
| `unless`      | Desugared in the parser → `If { cond: Unary(Not, ...) }` |
| `$"..."`      | `Token::InterpolStr(parts)` → `printf` with a constructed format string |
| `[...]`       | `alloca [N x i64]` + `getelementptr` with a computed index |
| `enum`        | `HashMap<enum, HashMap<variant, i64>>` in CodeGen |
| `defer`       | `Vec<Stmt>` — `emit_deferred()` before every `return` and at function end |
| `repeat N`    | Desugared in the parser → `Stmt::For` with hidden variable `__ri` |
| `? :`         | Phi node in LLVM (not `select`, to handle side effects correctly) |

### `save_and_compile(output, opts)` Pipeline

1. `module.print_to_file("<output>.ll")` — write LLVM IR
2. If `--emit-llvm` → stop
3. `llc <output>.ll -o <output>.s -relocation-model=pic`
4. `clang <output>.s -o <output> -lm`
5. Unless `--save-temps` → delete `.ll` and `.s`

---

## JVM Code Generator — `src/jvm/mod.rs`

Transpiles the AST to a `Main.java` source file, then invokes `javac` and `jar`.

```rust
pub struct JvmOptions {
    pub emit_java: bool,   // --emit-java: stop after generating .java
    pub verbose:   bool,
}

pub fn generate_and_compile(program: &[Stmt], output: &str, opts: &JvmOptions)
    -> Result<(), String>
```

**Orbitron → Java type mapping:**

| Orbitron | Java              |
|----------|-------------------|
| `int`    | `long`            |
| `float`  | `double`          |
| array    | `long[]`          |
| struct   | `static class`    |
| `self`   | `this`            |
| `init`   | Java constructor  |

**JVM backend details:**
- Enums: `EnumName_Variant` → static `long` constants
- `match`: unique variables `__m0`, `__m1`, ... per match block
- `defer`: try-finally block (LIFO order)
- Run: `java -jar <output>.jar`
- GraalVM: `native-image -jar <output>.jar`

---

## Standard Library — `stdlib/`

Written in Orbitron itself — no special compiler privileges.

| File       | Import              | Contents |
|------------|---------------------|----------|
| `math.ot`  | `import "std/math"` | abs, max, min, clamp, factorial, fib, gcd, lcm, sum_range, sign, is_prime; constants PI, E, INT_MAX |
| `bits.ot`  | `import "std/bits"` | bit_count, bit_len, is_pow2, next_pow2, prev_pow2, low_bit, shl, shr, floor_log2, reverse_bits |
| `algo.ot`  | `import "std/algo"` | min3, max3, median3, lerp, map_range, dist, digit_count, digit_sum, reverse_digits, is_palindrome_num, ipow, triangle, is_triangle, isqrt, is_square, near, cycle |

---

## Built-in Functions (declared in `CodeGen::new()`)

```c
int    printf(char*, ...);   // println
int    scanf(char*, ...);    // readInt / readFloat
double pow(double, double);  // ** operator
long   syscall(long, ...);   // syscall() builtin
```

---

## Dependencies (`Cargo.toml`)

| Crate      | Purpose                              |
|------------|--------------------------------------|
| `inkwell`  | LLVM IR generation (safe Rust API)   |
| `llvm-sys` | LLVM C bindings (transitive)         |
| `serde`    | Manifest deserialization             |
| `toml`     | TOML parsing                         |

LLVM version: 18.x (controlled via `inkwell` feature flags).

---

## Extending the Compiler

### Adding a new operator or syntax construct

1. **`token.rs`** — add a variant to `Token`; recognize it in `next_token()`
2. **`ast.rs`** — add a variant to `Stmt` or `Expr`
3. **`parser/mod.rs`** — parse it and construct the AST node
4. **`codegen/stmt.rs`** or **`codegen/expr.rs`** — emit LLVM IR
5. **`jvm/mod.rs`** — emit Java (if JVM backend support is needed)

### Adding a stdlib module

Create `stdlib/<name>.ot` with ordinary Orbitron functions and constants.
Users import it with `import "std/<name>"`.

### Adding a static type checker

Insert a pass between the parser and the code generator:

```
AST → TypeChecker → annotated AST → CodeGen
```
