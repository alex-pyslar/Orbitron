# Getting Started with Orbitron

This guide walks you through installing Orbitron, writing your first program, and understanding
the project structure.

---

## Prerequisites

### LLVM Backend (default)

| Tool    | Version | Install |
|---------|---------|---------|
| Rust    | 1.70+   | [rustup.rs](https://rustup.rs) |
| LLVM    | 18.x    | `sudo apt install llvm-18` (Ubuntu/Debian) |
| clang   | 18.x    | `sudo apt install clang-18` |
| libm    | any     | included in `libc-dev` |

> On Ubuntu 22.04+ you can install LLVM 18 via the LLVM apt repository:
> ```bash
> wget -O /tmp/llvm.sh https://apt.llvm.org/llvm.sh && sudo bash /tmp/llvm.sh 18
> ```

### JVM Backend (optional)

| Tool | Version |
|------|---------|
| JDK  | 11+     |

```bash
sudo apt install default-jdk
```

---

## Installation

### Build from Source

```bash
git clone https://github.com/alex-pyslar/Orbitron.git
cd Orbitron
cargo build --release
```

The compiler binary is placed at `target/release/orbitron`.

Add it to your `PATH` (optional but convenient):

```bash
export PATH="$PATH:$(pwd)/target/release"
```

Or on WSL (Windows Subsystem for Linux):
```bash
wsl -e bash -c "cd /mnt/c/source/Orbitron && cargo build --release 2>&1"
```

### Verify Installation

```bash
orbitron --version
orbitron --help
```

---

## Your First Program

### Single-file Mode

Create a file `hello.ot`:

```orbitron
func main() {
    println("Hello, World!");
    var x = 42;
    println($"x = {x}");
}
```

Compile and run:

```bash
orbitron hello.ot   # compiles to ./hello
./hello
```

Output:
```
Hello, World!
x = 42
```

### Project Mode

Create a new project:

```bash
orbitron new myapp
cd myapp
```

This generates:

```
myapp/
├── orbitron.toml
├── src/
│   └── main.ot
└── bin/
```

Edit `src/main.ot`, then:

```bash
orbitron run       # build + execute
orbitron build     # build only → bin/myapp
```

---

## Quick Language Tour

### Variables and Constants

```orbitron
var count = 0;          // mutable variable
const MAX: int = 100;   // compile-time constant
var pi: float = 3.14;   // optional type annotation
```

### Functions

```orbitron
func add(a: int, b: int): int {
    return a + b;
}

func main() {
    println(add(3, 4));   // 7
}
```

### Control Flow

```orbitron
if (x > 0) {
    println("positive");
} else {
    println("non-positive");
}

unless (x == 0) {         // Ruby-style negated if
    println(100 / x);
}

var label = x > 0 ? "pos" : "neg";  // ternary
```

### Loops

```orbitron
for i in 0..10 { println(i); }        // exclusive [0, 10)
for i in 0..=10 { println(i); }       // inclusive [0, 10]
while (n > 0) { n -= 1; }
repeat 5 { println("hi"); }           // exactly 5 times
loop { if (done) { break; } }         // infinite
```

### Arrays

```orbitron
var nums = [10, 20, 30, 40];
println(nums[0]);     // 10
nums[1] = 99;

var sum = 0;
for i in 0..4 { sum += nums[i]; }
```

### Structs and Classes

```orbitron
// Go/Rust style
struct Vec2 { x: int, y: int }
impl Vec2 {
    pub func len_sq(self): int { return self.x**2 + self.y**2; }
}
var v = Vec2 { x: 3, y: 4 };
println(v.len_sq());   // 25

// Java/C# style
class Counter {
    private val: int,
    init(start: int) { self.val = start; }
    pub func inc(self) { self.val += 1; }
    pub func get(self): int { return self.val; }
}
var c = new Counter(0);
c.inc();
println(c.get());   // 1
```

### String Interpolation, Enums, Defer, Pipe

```orbitron
var score = 95;
println($"Score: {score}");         // string interpolation

enum Grade { Fail, Pass, Good, Excellent }
var g = Grade.Excellent;            // g == 3

defer println("Done");              // runs at function exit

func double(n: int): int { return n * 2; }
var result = score |> double;       // 190
```

---

## Standard Library

```orbitron
import "std/math";

func main() {
    println(factorial(10));   // 3628800
    println(is_prime(97));    // 1
    println(gcd(48, 18));     // 6
}
```

Available modules: `std/math`, `std/bits`, `std/algo`, `std/sys`, `std/net`, `std/db`.

See [stdlib.md](stdlib.md) for the full reference.

---

## Compilation Backends

| Backend | Command                          | Output              |
|---------|----------------------------------|---------------------|
| LLVM    | `orbitron build`                 | native binary       |
| JVM     | `orbitron build --backend jvm`   | `<output>.jar`      |

Switch the default in `orbitron.toml`:

```toml
[build]
backend = "jvm"
```

---

## Next Steps

| Topic                          | Document |
|--------------------------------|----------|
| Full language reference        | [syntax.md](syntax.md) |
| Standard library reference     | [stdlib.md](stdlib.md) |
| Low-level / systems programming| [lowlevel.md](lowlevel.md) |
| Compiler internals             | [architecture.md](architecture.md) |
| Annotated examples             | [examples.md](examples.md) |
