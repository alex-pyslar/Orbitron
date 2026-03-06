# Orbitron

**A compiled, multi-backend programming language with clean, expressive syntax.**

Orbitron compiles `.ot` source files to native binaries via LLVM IR, or to cross-platform JVM bytecode (`.jar`). Its syntax blends the best ideas from Go, Rust, Python, Ruby, Elixir, C#, Kotlin, and Java into a single coherent language.

```orbitron
func main() {
    var name = "Orbitron";
    var version = 2;
    println($"Welcome to {name} v{version}!");

    var primes = [2, 3, 5, 7, 11];
    var sum = 0;
    for i in 0..5 { sum += primes[i]; }
    println($"Sum of first 5 primes: {sum}");
}
```

---

## Features

| Feature | Syntax | Origin |
|---------|--------|--------|
| Variables | `var x = 42;` | |
| Constants | `const MAX: int = 100;` | Rust, C++ |
| String interpolation | `$"val = {x}"` | C#, Kotlin |
| Power operator | `2 ** 10` | Python |
| Pipe operator | `x \|> double \|> inc` | Elixir, F# |
| `unless` conditional | `unless (x == 0) { }` | Ruby |
| Arrays | `var a = [1, 2, 3];` | Python, JS |
| Enums | `enum Dir { North, South }` | Rust, Swift |
| Deferred execution | `defer println("done");` | Go |
| Repeat N times | `repeat 5 { }` | Lua, Pascal |
| Ternary operator | `a > b ? a : b` | C, Java |
| Structs + methods | `struct Foo { } impl Foo { }` | Go, Rust |
| Classes | `class Foo { init() { } }` | Java, Kotlin |
| Raw syscalls | `syscall(SYS_WRITE, STDOUT, buf, n);` | C |
| Extern C | `extern func socket(...): int;` | C |

---

## Quick Start

```bash
# Build the compiler (requires Rust + LLVM 18)
cargo build --release

# Compile and run a single file
./target/release/orbitron examples/hello.ot && ./hello

# Create a new project
./target/release/orbitron new myapp
cd myapp
../target/release/orbitron run
```

**On Windows — use WSL:**

```bash
wsl -e bash -c "cd /mnt/c/source/Orbitron && cargo build --release 2>&1"
wsl -e bash -c "cd /mnt/c/source/Orbitron && ./target/release/orbitron examples/hello.ot && ./hello"
```

---

## Installation

### Prerequisites

| Tool | Purpose |
|------|---------|
| Rust + Cargo | Build the compiler |
| LLVM 18 (`llc`, `clang`) | LLVM backend — native binary output |
| JDK 11+ (`javac`, `jar`) | JVM backend — `.jar` output |
| `libm` | Math operations (`**` operator) |

### Build from Source

```bash
git clone https://github.com/alex-pyslar/Orbitron
cd Orbitron
cargo build --release
```

The compiler binary is `target/release/orbitron`.

The standard library is in `stdlib/`. It must be next to the binary, or set `ORBITRON_HOME`:

```bash
export ORBITRON_HOME=/path/to/Orbitron
```

---

## Language Tour

### Variables and Types

```orbitron
var x = 42;           // int (i64)
var pi: float = 3.14; // float (f64) — type annotation optional
var flag = true;      // true == 1, false == 0

const MAX: int   = 1000;
const TAX: float = 0.2;
```

### Functions

```orbitron
func add(a: int, b: int): int {
    return a + b;
}

func main() {
    println(add(10, 20));  // 30
}
```

### Control Flow

```orbitron
if (score >= 90) {
    println("Excellent");
} else if (score >= 70) {
    println("Good");
} else {
    println("Keep going");
}

// unless — runs when condition is FALSE (inspired by Ruby)
unless (x == 0) {
    println(100 / x);  // safe division
}

// match — pattern matching on integers and enums
enum Status { Ok, Error, Pending }
var s = Status.Ok;

match s {
    Status.Ok      => { println("all good"); }
    Status.Error   => { println("error!");   }
    Status.Pending => { println("waiting");  }
    _              => { println("unknown");  }
}
```

### Loops

```orbitron
for i in 0..10 { }         // exclusive range: 0 to 9
for i in 0..=10 { }        // inclusive range: 0 to 10
for i in 0..3, j in 0..3 { }   // nested loops in one line

while (n > 0) { n -= 1; }

do { n += 1; } while (n < 10);

loop { if (done) { break; } }

repeat 5 { counter += 1; }   // exactly 5 times (from Lua / Pascal)
```

### Arrays

```orbitron
var primes = [2, 3, 5, 7, 11, 13];

println(primes[0]);    // 2
primes[0] = 99;        // mutate

var sum = 0;
for i in 0..6 { sum += primes[i]; }
println(sum);          // 138
```

### String Interpolation

```orbitron
var score = 95;
var name = 42;
println($"Score for player {name}: {score}");
```

### Pipe Operator

```orbitron
func double(n: int): int { return n * 2; }
func inc(n: int):    int { return n + 1; }
func square(n: int): int { return n * n; }

// left-to-right function composition
var result = 3 |> double |> inc |> square;  // ((3*2)+1)^2 = 49
println(result);
```

### Structs — Go / Rust Style

```orbitron
struct Point {
    x: int,
    y: int,
}

impl Point {
    pub func dist_sq(self): int {
        return self.x * self.x + self.y * self.y;
    }

    pub func move_by(self, dx: int, dy: int) {
        self.x = self.x + dx;
        self.y = self.y + dy;
    }
}

// struct literal — no `new` keyword
var p = Point { x: 3, y: 4 };
println(p.dist_sq());   // 25
p.move_by(1, -1);
println(p.dist_sq());   // 13
```

### Classes — Java / Kotlin Style

```orbitron
class BankAccount {
    private balance: int,

    init(initial: int) {
        self.balance = initial;
    }

    pub func deposit(self, amount: int) {
        if (amount > 0) { self.balance += amount; }
    }

    pub func withdraw(self, amount: int): int {
        if (self.balance >= amount) {
            self.balance -= amount;
            return 1;
        }
        return 0;  // insufficient funds
    }

    pub func get_balance(self): int {
        return self.balance;
    }
}

var acc = new BankAccount(500);
acc.deposit(200);
println(acc.get_balance());   // 700
println(acc.withdraw(300));   // 1 (success)
println(acc.get_balance());   // 400
```

### Defer

```orbitron
func process() {
    defer println("cleanup");   // always runs last (LIFO order)
    defer println("closing");   // runs second-to-last

    println("working...");
    // Output order: working... → closing → cleanup
}
```

### Standard Library

```orbitron
import "std/math";
import "std/bits";
import "std/algo";

func main() {
    println(factorial(10));          // 3628800
    println(gcd(48, 18));            // 6
    println(is_prime(97));           // 1
    println(sum_range(1, 100));      // 5050

    println(bit_count(255));         // 8
    println(is_pow2(16));            // 1
    println(next_pow2(5));           // 8

    println(ipow(2, 10));            // 1024
    println(isqrt(144));             // 12
    println(is_palindrome_num(121)); // 1
}
```

---

## CLI Reference

```
USAGE:
  orbitron new <name>            Create a new project
  orbitron build [options]       Build project (reads orbitron.toml)
  orbitron run   [options]       Build and run project
  orbitron <file.ot> [options]   Compile a single file

OPTIONS:
  -h, --help              Show help and exit
      --version           Show version and exit
  -o <file>               Output file name
      --backend llvm|jvm  Compilation backend (default: llvm)
      --emit-llvm         Save LLVM IR (.ll) and stop
      --emit-java         Save Java source (.java) and stop
      --save-temps        Keep intermediate files (.ll, .s)
  -v, --verbose           Print compilation steps

BACKENDS:
  llvm   -> native binary  (requires llc + clang + libm)
  jvm    -> .jar file      (requires javac + jar; run with java -jar)
```

**Examples:**

```bash
orbitron new myapp                # scaffold project
cd myapp && orbitron run          # build + run (LLVM)
orbitron hello.ot                 # single file (LLVM)
orbitron hello.ot --backend jvm   # single file (JVM)
orbitron build --emit-llvm        # inspect LLVM IR
orbitron build -v                 # verbose output
```

---

## Project Configuration

```
myproject/
├── orbitron.toml
├── src/
│   ├── main.ot       # entry point — must contain func main()
│   └── utils.ot      # module (import "utils";)
└── bin/              # compiled output
```

**orbitron.toml:**

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

## Standard Library

| Module | Import | What's inside |
|--------|--------|--------------|
| math | `import "std/math"` | `abs`, `max`, `min`, `clamp`, `factorial`, `fib`, `gcd`, `lcm`, `sign`, `is_prime`, `PI`, `E`, `INT_MAX` |
| bits | `import "std/bits"` | `bit_count`, `bit_len`, `is_pow2`, `next_pow2`, `prev_pow2`, `shl`, `shr`, `floor_log2`, `reverse_bits` |
| algo | `import "std/algo"` | `min3`, `max3`, `median3`, `lerp`, `map_range`, `dist`, `ipow`, `isqrt`, `digit_sum`, `is_palindrome_num`, `cycle` |
| sys  | `import "std/sys"` | Linux syscall constants, `sys_alloc`, `sys_free`, `sys_write`, `sys_sleep`, `sys_getpid` |
| net  | `import "std/net"` | `tcp_socket`, `tcp_connect`, `net_send`, `net_recv`, `net_bind`, `net_listen`, `net_accept` |

---

## Low-Level Programming

Orbitron gives you pointer arithmetic, raw Linux syscalls, and extern C function declarations:

```orbitron
import "std/sys";

func main() {
    // Heap allocation
    var buf = sys_alloc(64);

    // Write bytes by address
    ptr_write_byte(buf,     79);   // 'O'
    ptr_write_byte(buf + 1, 114);  // 'r'
    ptr_write_byte(buf + 2, 98);   // 'b'
    ptr_write_byte(buf + 3, 10);   // '\n'

    // Raw Linux syscall: write(stdout, buf, 4)
    syscall(SYS_WRITE, STDOUT, buf, 4);

    sys_free(buf, 64);
}
```

```orbitron
// Declare external C functions
extern func open(path: int, flags: int): int;
extern func read(fd: int, buf: int, n: int): int;
extern func close(fd: int): int;
```

---

## Examples

| File | Topic |
|------|-------|
| [`examples/hello.ot`](examples/hello.ot) | Hello World |
| [`examples/variables.ot`](examples/variables.ot) | Variables, types, constants, interpolation |
| [`examples/control_flow.ot`](examples/control_flow.ot) | if / else / unless / match |
| [`examples/loops.ot`](examples/loops.ot) | All loop constructs |
| [`examples/functions.ot`](examples/functions.ot) | Functions, recursion, pipe operator |
| [`examples/arrays.ot`](examples/arrays.ot) | Arrays — creation, mutation, iteration |
| [`examples/structs.ot`](examples/structs.ot) | struct + impl (Go/Rust OOP) |
| [`examples/classes.ot`](examples/classes.ot) | class + init (Java/Kotlin OOP) |
| [`examples/enums.ot`](examples/enums.ot) | Enums and pattern matching |
| [`examples/features.ot`](examples/features.ot) | All 10 language features |
| [`examples/fibonacci.ot`](examples/fibonacci.ot) | Recursion and iteration |
| [`examples/sorting.ot`](examples/sorting.ot) | Bubble sort with arrays |
| [`examples/stdlib_demo.ot`](examples/stdlib_demo.ot) | Standard library showcase |
| [`examples/syscall_demo.ot`](examples/syscall_demo.ot) | Pointers and raw syscalls |
| [`examples/net_demo.ot`](examples/net_demo.ot) | TCP networking |
| [`examples/projects/calculator/`](examples/projects/calculator/) | Multi-file project |
| [`examples/projects/geometry/`](examples/projects/geometry/) | Multi-file project with imports |

---

## Compilation Pipeline

```
┌─────────────────────────────────────────────────────────────┐
│                         LLVM Backend                         │
│                                                             │
│  source.ot  ─►  Lexer  ─►  Parser  ─►  Resolver           │
│                                            │                │
│                              Merged AST ◄──┘                │
│                                  │                          │
│                              CodeGen  ─►  LLVM IR (.ll)     │
│                                               │             │
│                                  llc ◄────────┘             │
│                                   │                         │
│                                Assembly (.s)                │
│                                   │                         │
│                               clang -lm                     │
│                                   │                         │
│                             Native Binary                   │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                         JVM Backend                          │
│                                                             │
│  source.ot ─► Lexer ─► Parser ─► Resolver ─► JvmCodeGen    │
│                                                    │        │
│                              Main.java ◄───────────┘        │
│                                  │                          │
│                              javac + jar                    │
│                                  │                          │
│                             output.jar                      │
└─────────────────────────────────────────────────────────────┘
```

---

## Repository Layout

```
src/
├── main.rs          CLI dispatcher
├── cli.rs           Argument parser, help text, Backend enum
├── pipeline.rs      LLVM and JVM compilation pipelines
├── error.rs         CompileError type
├── project.rs       orbitron.toml manifest reader
├── resolver.rs      Recursive import resolver and AST merger
├── lexer/
│   ├── mod.rs       Lexer — tokenizes .ot source
│   └── token.rs     Token enum and keyword table
├── parser/
│   ├── mod.rs       Recursive descent parser
│   └── ast.rs       AST node types (Expr, Stmt, ...)
├── codegen/
│   ├── mod.rs       CodeGen struct, 3-pass code generation
│   ├── expr.rs      Expression → LLVM IR
│   └── stmt.rs      Statement → LLVM IR
└── jvm/
    └── mod.rs       AST → Java source → javac → jar

stdlib/               Standard library (written in Orbitron itself)
docs/                 Language reference and guides
examples/             Example programs, organized by topic
```

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for how to add language features, fix bugs, or extend the standard library.

## License

MIT — see [LICENSE](LICENSE) for details.
