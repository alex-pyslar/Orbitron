# Orbitron — Claude Code Instructions

## Project Overview

Orbitron is a **compiled programming language** with `.ot` file extension.
Compiler is written in **Rust**. Supports two backends:
- **LLVM** — native binaries via LLVM IR → `llc` → `clang`
- **JVM** — cross-platform `.jar` via AST → Java source → `javac`

Repository: `github.com/alex-pyslar/Orbitron`

---

## Repository Layout

```
src/
├── main.rs          CLI dispatcher
├── cli.rs           Argument parsing, Backend enum
├── pipeline.rs      Compile pipelines (LLVM + JVM)
├── resolver.rs      Recursive import resolver
├── error.rs         Error types
├── lexer/
│   ├── mod.rs       Lexer
│   └── token.rs     Token enum + InterpolPart
├── parser/
│   ├── mod.rs       Recursive-descent parser
│   └── ast.rs       AST nodes (Expr, Stmt, ...)
├── codegen/
│   ├── mod.rs       CodeGen, 3-pass code generation
│   ├── expr.rs      Expressions → LLVM IR
│   └── stmt.rs      Statements → LLVM IR
├── jvm/
│   └── mod.rs       AST → Java → javac → jar
├── fmt/
│   └── mod.rs       AST pretty-printer (orbitron fmt)
└── project.rs       orbitron.toml manifest loader

stdlib/              Standard library (.ot source)
docs/                Language book (Russian, 12 chapters)
examples/            Topic-organised examples (01–08)
vscode-orbitron/     VS Code language extension
```

---

## Language Syntax Quick Reference

### Variables & Constants
```orbitron
var x = 42;              // immutable (i64 inferred)
var mut count = 0;       // mutable
var pi: f64 = 3.14;      // type-annotated
const MAX: i64 = 1000;   // compile-time constant
```

### Functions
```orbitron
fn add(a: i64, b: i64) -> i64 { return a + b; }
fn double(n: i64) => n * 2;          // expression body
async fn fetch(n: i64) -> i64 { ... }
```

### Control Flow
```orbitron
if (cond) { } else { }
unless (cond) { }           // inverted if (Ruby)
match val { A => { } _ => { } }
for i in 0..10 { }          // exclusive range
for i in 0..=10 { }         // inclusive range
while (cond) { }
do { } while (cond);
repeat 5 { }                // repeat N times
defer expr;                 // LIFO deferred execution (Go)
```

### OOP
```orbitron
// Struct (Go/Rust style)
struct Point { public var x: i64, private var y: i64, static var count: i64 }
impl Point { public static fn new(...) -> Point { ... } }
Point::new(1, 2);            // static call via ::

// Class (Java/Kotlin style)
class Foo { init(n: i64) { self.n = n; } }
var f = new Foo(42);

// Trait
trait Drawable { fn draw(self); }
impl Drawable for Circle { fn draw(self) { ... } }
```

### Concurrency
```orbitron
go { println("goroutine"); };       // goroutine (Go)
var ch = chan();
go { ch <- 42; };
var v = <-ch;

async fn work() -> i64 { ... }
var result = await work();
```

### String Interpolation
```orbitron
var s = $"Hello, {name}! version={ver}";
```

### Operators
| Operator | Meaning               |
|----------|-----------------------|
| `**`     | Power (Python)        |
| `\|>`    | Pipe (Elixir/F#)      |
| `?:`     | Elvis / null-coalesce |
| `?.`     | Optional chaining     |
| `<-`     | Channel send/receive  |
| `::`     | Static namespace      |
| `=>`     | Match arm / expr body |

### Access Modifiers
`public` · `private` · `protected` · `internal` · `static`

### Standard Library
```orbitron
import "std/math";   // abs, max, min, factorial, gcd, is_prime, PI, E
import "std/bits";   // bit_count, bit_len, is_pow2, next_pow2
import "std/algo";   // ipow, isqrt, lerp, map_range, digit_sum
import "std/sys";    // Linux syscall constants, sys_alloc/free/write
import "std/net";    // tcp_connect/listen/accept, net_send/recv
import "std/db";     // SQLite3 bindings (requires -lsqlite3)
```

---

## CLI Commands

```
orbitron new <name>          New project (creates orbitron.toml + src/main.ot)
orbitron build [opts]        Build project from orbitron.toml
orbitron run   [opts]        Build + run
orbitron fmt   [files]       Format source files
orbitron <file.ot> [opts]    Compile single file

Options:
  -o <file>              Output file name
  --backend llvm|jvm     Backend (default: llvm)
  --emit-llvm            Save LLVM IR and stop
  --emit-java            Save Java source and stop
  --save-temps           Keep intermediate files
  -v, --verbose          Verbose output
```

---

## Build & Run (Windows — requires WSL)

```bash
# Build compiler
wsl -e bash -c "cd /mnt/c/source/Orbitron && cargo build --release 2>&1"

# Run single file
wsl -e bash -c "cd /mnt/c/source/Orbitron && ./target/release/orbitron examples/01_basics/hello.ot && ./hello"

# Debug build
wsl -e bash -c "cd /mnt/c/source/Orbitron && cargo build 2>&1 | tail -20"

# Tests
wsl -e bash -c "cd /mnt/c/source/Orbitron && cargo test 2>&1"
```

LLVM 18 must be installed in WSL. JDK 11+ required for `--backend jvm`.

---

## Code Style

- Rust code follows standard `rustfmt` conventions
- Orbitron source uses 4-space indentation
- Comments in Russian are normal throughout the codebase
- `#![allow(dead_code)]` at crate root works around a rustc ICE near Cyrillic comments
- Prefer `Err(format!(...))` for error propagation via `?` — no `unwrap()` in production paths

---

## Architecture Notes

### Compilation Pipeline (LLVM)
```
.ot source
  → Lexer (token.rs)
  → Parser (ast.rs)
  → Resolver (resolves imports recursively, merges ASTs)
  → CodeGen (3-pass: forward-declare structs → functions → bodies)
  → LLVM IR (.ll)
  → llc → .s
  → clang -lm → binary
```

### Compilation Pipeline (JVM)
```
.ot source → Lexer → Parser → Resolver → JvmCodeGen
  → Main.java → javac → jar
```

### Key Invariants
- `Resolver` handles `import "path"` by finding `.ot` files relative to `ORBITRON_HOME/stdlib/` or project root
- `CodeGen` uses a **3-pass** approach to handle forward references (struct types → fn signatures → fn bodies)
- `Token::InterpolStr` carries pre-parsed `Vec<InterpolPart>` — literal segments and variable holes

---

## Project File: orbitron.toml

```toml
[project]
name    = "myapp"
version = "0.1.0"

[build]
main    = "src/main.ot"
output  = "bin/myapp"
backend = "llvm"          # or "jvm"
```

---

## Common Tasks

| Task | Command |
|------|---------|
| Add keyword to lexer | Edit `src/lexer/token.rs` + `src/lexer/mod.rs` keyword map |
| Add AST node | Edit `src/parser/ast.rs`, then update parser + codegen |
| Add stdlib module | Add `.ot` file to `stdlib/`, register in `resolver.rs` |
| Add example | Create `examples/NN_topic/name.ot` |
| Update docs | Edit `docs/chNN_*.md`, regenerate `docs/SUMMARY.md` |
| Format all examples | `orbitron fmt examples/**/*.ot` |

---

## Important Files

| File | Role |
|------|------|
| `src/lexer/token.rs` | Complete token enum — source of truth for all keywords |
| `src/parser/ast.rs` | Complete AST — source of truth for all syntax constructs |
| `src/codegen/mod.rs` | Top-level codegen, struct/function forward-declaration passes |
| `src/pipeline.rs` | End-to-end compile pipelines for both backends |
| `Cargo.toml` | Crate metadata, dependencies |
| `docs/reference.md` | Comprehensive language reference |
