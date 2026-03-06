# Contributing to Orbitron

Thank you for your interest in contributing! This document explains how to get
the project building locally, where each piece of functionality lives, and the
conventions used throughout the codebase.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Building from Source](#building-from-source)
3. [Running the Tests](#running-the-tests)
4. [Project Layout](#project-layout)
5. [How to Add a Language Feature](#how-to-add-a-language-feature)
6. [How to Add a Standard Library Module](#how-to-add-a-standard-library-module)
7. [Coding Conventions](#coding-conventions)
8. [Submitting a Pull Request](#submitting-a-pull-request)
9. [Reporting Bugs](#reporting-bugs)

---

## Prerequisites

| Tool   | Version | Notes                                              |
|--------|---------|----------------------------------------------------|
| Rust   | 1.70+   | Install via [rustup.rs](https://rustup.rs)         |
| LLVM   | 18.x    | `sudo apt install llvm-18 clang-18` (Ubuntu/Debian)|
| JDK    | 11+     | Optional — required only for the JVM backend       |

On Ubuntu 22.04+ you can install LLVM 18 via the official LLVM apt repository:

```bash
wget -O /tmp/llvm.sh https://apt.llvm.org/llvm.sh && sudo bash /tmp/llvm.sh 18
```

On Windows, use WSL2 with Ubuntu 22.04+ for the full toolchain.

---

## Building from Source

```bash
git clone https://github.com/alex-pyslar/Orbitron.git
cd Orbitron
cargo build --release
```

The compiler binary is placed at `target/release/orbitron`.

### Verify the build

```bash
./target/release/orbitron examples/hello.ot -o /tmp/hello
/tmp/hello
# → Hello, World!
```

---

## Running the Tests

There is no automated test suite yet — this is a great area to contribute!

Until a test harness is added, the recommended workflow is:

1. Build the compiler.
2. Compile each example with the LLVM backend and verify the output:

```bash
cargo build --release 2>&1

for f in examples/*.ot; do
    echo "--- $f ---"
    ./target/release/orbitron "$f" -o /tmp/orb_test && /tmp/orb_test
done
```

3. Compile a few examples with the JVM backend:

```bash
./target/release/orbitron examples/fibonacci.ot --backend jvm -o /tmp/fib
java -jar /tmp/fib.jar
```

If you are adding a new feature, please also add or update an example file that
exercises it so reviewers can quickly verify the behaviour.

---

## Project Layout

```
src/
├── main.rs          — CLI dispatcher (new / build / run / <file.ot>)
├── cli.rs           — BuildOpts, Backend enum, parse_build_opts()
├── pipeline.rs      — compile_llvm(), compile_jvm(), find_stdlib()
├── error.rs         — CompileError type
├── project.rs       — orbitron.toml manifest (serde + toml)
├── resolver.rs      — recursive import resolver (AST merger)
├── lexer/
│   ├── mod.rs       — struct Lexer, tokenize()
│   └── token.rs     — enum Token, keyword table
├── parser/
│   ├── mod.rs       — recursive-descent parser
│   └── ast.rs       — Expr, Stmt, BinOp, UnaryOp, ...
├── codegen/
│   ├── mod.rs       — CodeGen struct, generate_program(), save_and_compile()
│   ├── expr.rs      — gen_expr(), gen_binop()
│   └── stmt.rs      — gen_stmt() for every Stmt variant
└── jvm/
    └── mod.rs       — JvmCodeGen, generate_and_compile()

stdlib/              — Standard library written in Orbitron itself
├── math.ot
├── bits.ot
├── algo.ot
├── sys.ot
├── net.ot
└── db.ot

examples/            — Annotated example programs (.ot)
docs/                — Reference documentation (Markdown)
```

See [docs/architecture.md](docs/architecture.md) for a detailed description of
every module and the compilation pipeline.

---

## How to Add a Language Feature

Adding a new operator or syntax construct requires touching up to five files.
Follow these steps in order:

### 1. `src/lexer/token.rs` — add a Token variant

```rust
pub enum Token {
    // ... existing tokens ...
    MyNewOp,   // add your variant here
}
```

Recognise it inside `Lexer::next_token()` in `src/lexer/mod.rs`:

```rust
'@' => { self.advance(); Token::MyNewOp }
```

### 2. `src/parser/ast.rs` — add an AST node

For a new expression:

```rust
pub enum Expr {
    // ... existing variants ...
    MyNew(Box<Expr>),
}
```

For a new statement:

```rust
pub enum Stmt {
    // ... existing variants ...
    MyNewStmt { expr: Expr },
}
```

### 3. `src/parser/mod.rs` — parse the new construct

Insert a parsing call at the correct precedence level (see the existing
`parse_pipe` → `parse_ternary` → … → `parse_primary` chain).

### 4. `src/codegen/expr.rs` or `src/codegen/stmt.rs` — emit LLVM IR

Extend `gen_expr()` or `gen_stmt()` with a new match arm:

```rust
Expr::MyNew(inner) => {
    let val = self.gen_expr(inner, func, bb)?;
    // ... emit LLVM IR ...
    Ok(val)
}
```

### 5. `src/jvm/mod.rs` — emit Java (optional)

If the feature should work with `--backend jvm`, add a corresponding arm in the
JVM code generator. If it is LLVM-only, add a `panic!` with a clear message:

```rust
Expr::MyNew(_) => panic!("MyNew is not supported in the JVM backend"),
```

---

## How to Add a Standard Library Module

Standard library modules are plain Orbitron source files — no special compiler
support is needed.

1. Create `stdlib/<name>.ot` with your functions and constants.
2. Users import it with `import "std/<name>";`.
3. Document it in [docs/stdlib.md](docs/stdlib.md).
4. Add a small demo to `examples/` if the module is non-trivial.

**Important constraints:**

- Do not define a function named `pow` — it conflicts with the pre-declared
  libm `pow(double, double)` used by the `**` operator.
- All parameters and return values are `i64`. There is no float-typed stdlib
  parameter support yet.
- Arrays cannot currently be passed to or returned from stdlib functions.

---

## Coding Conventions

- **Rust edition**: 2021.
- **Error messages**: all user-visible strings (panics, errors, verbose output)
  must be in English.
- **Naming**: follow standard Rust conventions (`snake_case` for functions and
  variables, `CamelCase` for types).
- **Orbitron source files**: use ASCII-only identifiers; Cyrillic identifiers
  are not supported by the lexer.
- **Comments in `.ot` files**: English only.
- **No dead code warnings**: if the compiler emits `dead_code` warnings under
  rustc 1.93+, add `#![allow(dead_code)]` at the top of `main.rs` (already
  present).

---

## Submitting a Pull Request

1. Fork the repository and create a feature branch:

   ```bash
   git checkout -b feature/my-feature
   ```

2. Make your changes and verify that all existing examples still compile and
   produce correct output (see [Running the Tests](#running-the-tests)).

3. Add or update an example or a doc section that covers the new behaviour.

4. Commit with a descriptive message:

   ```bash
   git commit -m "Add: repeat-N loop desugared to Stmt::For"
   ```

5. Open a pull request against `main`. Describe **what** changed and **why**.

---

## Reporting Bugs

Please open an issue on GitHub and include:

- The `.ot` source file that triggers the bug (or a minimal reproduction).
- The exact command you ran (`orbitron <file.ot> --backend llvm`, etc.).
- The full error message or unexpected output.
- Your OS, Rust version (`rustc --version`), and LLVM version (`llc --version`).
