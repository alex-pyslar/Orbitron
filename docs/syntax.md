# Orbitron Language Reference

Orbitron is a compiled language with syntax inspired by Go, Rust, Python, Ruby, Elixir, Java, C#,
and Kotlin. It compiles to a native binary via LLVM IR, or to JVM bytecode (`.jar`).

---

## Table of Contents

1. [Variables and Types](#1-variables-and-types)
2. [Constants](#2-constants)
3. [Functions](#3-functions)
4. [Output and Input](#4-output-and-input)
5. [String Interpolation](#5-string-interpolation)
6. [Operators](#6-operators)
7. [Conditionals](#7-conditionals)
8. [Loops](#8-loops)
9. [Arrays](#9-arrays)
10. [Enums](#10-enums)
11. [Pattern Matching](#11-pattern-matching)
12. [Defer](#12-defer)
13. [Structs](#13-structs)
14. [Classes](#14-classes)
15. [Project System and Imports](#15-project-system-and-imports)
16. [Standard Library](#16-standard-library)
17. [Compilation Backends](#17-compilation-backends)
18. [Operator Precedence](#18-operator-precedence)
19. [Grammar (EBNF)](#19-grammar-ebnf)

---

## 1. Variables and Types

```orbitron
var x = 42;            // integer (int, 64-bit)
var pi: float = 3.14;  // float (64-bit); type annotation is optional
var s = 10;
s = s + 1;             // reassignment without var
```

Supported types: `int` (i64), `float` (f64).

In mixed arithmetic expressions, `int` is automatically promoted to `float`.

---

## 2. Constants  *(Rust / C++)*

```orbitron
const MAX: int   = 100;
const PI:  int   = 3;
const TAX: float = 0.2;

func main() {
    println(MAX);          // 100
    println(PI * 5 * 5);   // 75
}
```

- May be declared at the top level or inside a function.
- Value must be a numeric literal.
- Visible to all functions in the same file and in importing files.

---

## 3. Functions

```orbitron
func add(a: int, b: int): int {
    return a + b;
}

func greet() {
    println("Hello!");
}
```

Type annotations on parameters and the return type are **optional** (they serve as documentation).

The program entry point is the `main` function:

```orbitron
func main() {
    println(add(2, 3)); // 5
}
```

---

## 4. Output and Input

| Construct      | Description                          |
|----------------|--------------------------------------|
| `println(val);`| Print a value followed by a newline  |
| `readInt()`    | Read one integer from stdin          |
| `readFloat()`  | Read one float from stdin            |

```orbitron
println("Enter a number:");
var n = readInt();
println(n * n);

var f = readFloat();
println(f * 2.0);
```

---

## 5. String Interpolation  *(C# / Kotlin)*

The `$"..."` syntax embeds variables and constants directly into a string:

```orbitron
var x     = 42;
var score = 100;
println($"x = {x}");           // x = 42
println($"score: {score}");    // score: 100
println($"PI = {PI}");         // PI = 3  (constant)
```

> Supported: `int` and `float` variables and constants.
> String interpolation is only allowed inside `println()`.

---

## 6. Operators

### Arithmetic

| Operator | Meaning                              |
|----------|--------------------------------------|
| `+`      | Addition                             |
| `-`      | Subtraction                          |
| `*`      | Multiplication                       |
| `/`      | Division                             |
| `%`      | Remainder                            |
| `**`     | Exponentiation *(Python)*            |

```orbitron
var p = 2 ** 10;    // 1024
var q = 3 ** 4;     // 81
```

### Comparison

`==`  `!=`  `<`  `<=`  `>`  `>=`

Result: `-1` (true) or `0` (false) — both are represented as `int`.

### Logical

`&&`  `||`  `!`

### Ternary Operator  *(C / Java)*

```orbitron
var max = a > b ? a : b;
var abs = x >= 0 ? x : -x;

// Chained (right-associative):
var label = n > 10 ? 3 : n > 0 ? 2 : 1;
```

### Pipe Operator `|>`  *(Elixir / F#)*

Passes the left-hand value as the first argument to the right-hand function:

```orbitron
func double(n: int): int { return n * 2; }
func inc(n: int):    int { return n + 1; }

var result = 3 |> double |> inc;   // inc(double(3)) = 7
```

### Compound Assignment

| Form       | Equivalent    |
|------------|---------------|
| `x += 5;`  | `x = x + 5;`  |
| `x -= 3;`  | `x = x - 3;`  |
| `x *= 2;`  | `x = x * 2;`  |
| `x /= 4;`  | `x = x / 4;`  |

---

## 7. Conditionals

```orbitron
if (condition) {
    // ...
} else if (other) {
    // ...
} else {
    // ...
}
```

### `unless`  *(Ruby)*

Executes when the condition is **false** — syntactic sugar for `if (!...)`:

```orbitron
unless (x == 0) {
    println(100 / x);   // safe division
}
```

---

## 8. Loops

### `for..in` — range loop

```orbitron
// Exclusive range [from, to)
for i in 0..4 {
    println(i);   // 0 1 2 3
}

// Inclusive range [from, to]
for i in 0..=4 {
    println(i);   // 0 1 2 3 4
}
```

### Multi-range `for` — nested loops on one line

```orbitron
// Equivalent to: for i { for j { ... } }
for i in 0..3, j in 0..3 {
    println(i * 10 + j);
}
```

### `while` — pre-condition loop

```orbitron
while (n > 0) {
    n -= 1;
}
```

### `do..while` — post-condition loop

```orbitron
do {
    n += 1;
} while (n < 10);
```

### `loop` — infinite loop

```orbitron
loop {
    if (done) { break; }
}
```

### `repeat N`  *(Lua / Pascal)*

Execute the body exactly N times:

```orbitron
repeat 5 {
    println("Hello!");
}

var counter = 0;
repeat 10 { counter += 1; }
// counter == 10
```

### `break` and `continue`

```orbitron
for i in 0..10 {
    if (i == 5)      { break; }
    if (i % 2 == 0)  { continue; }
    println(i);    // 1 3
}
```

---

## 9. Arrays  *(Python / JavaScript)*

```orbitron
var primes = [2, 3, 5, 7, 11];

// Read
println(primes[0]);    // 2
println(primes[4]);    // 11

// Write
primes[0] = 99;

// Iterate
for i in 0..5 {
    println(primes[i]);
}

// Accumulate
var sum = 0;
for i in 0..5 { sum += primes[i]; }
```

Array elements are `int`. The size is determined at initialization.

---

## 10. Enums  *(Rust / Swift)*

```orbitron
enum Color  { Red, Green, Blue }
enum Season { Spring, Summer, Autumn, Winter }
```

Each variant receives an integer value: 0, 1, 2, ...

```orbitron
var c = Color.Red;      // c == 0
var s = Season.Summer;  // s == 1
println(s);             // 1
```

---

## 11. Pattern Matching

```orbitron
match expression {
    value        => { /* block */ }
    EnumName.Var => { /* enum variant */ }
    _            => { /* wildcard / default */ }
}
```

Patterns: integer literals, enum variants, `_` (wildcard).

```orbitron
enum Dir { North, South, East, West }
var d = Dir.East;

match d {
    Dir.North => { println("North"); }
    Dir.East  => { println("East"); }
    _         => { println("Other"); }
}
```

---

## 12. Defer  *(Go)*

`defer` registers a statement to execute **just before the function returns**.
Multiple defers execute in LIFO order (last registered, first executed).

```orbitron
func example() {
    defer println("Done!");    // runs last
    println("Start");
    println("Middle");
}
// Output: Start → Middle → Done!
```

---

## 13. Structs  *(Go / Rust style)*

Data and methods are defined separately.

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

// Creation: struct literal — no `new`
var p = Point { x: 3, y: 4 };
println(p.dist_sq());   // 25
p.move_by(1, 0);
```

Fields: `name: int` or `name: float`.
`self` is an explicit first parameter for all methods.

---

## 14. Classes  *(Java / C# / Kotlin style)*

Data and methods are co-located; a constructor is declared with `init`.

```orbitron
class Counter {
    private val:  int,
    private step: int,

    init(v: int, s: int) {
        self.val  = v;
        self.step = s;
    }

    pub func tick(self) {
        self.val = self.val + self.step;
    }

    pub func get(self): int {
        return self.val;
    }
}

var c = new Counter(0, 5);
c.tick();
println(c.get());   // 5
```

### Access Modifiers

| Keyword   | Meaning                    |
|-----------|----------------------------|
| `pub`     | Public (default)           |
| `private` | Private                    |

### Comparing OOP Styles

| Aspect      | `struct + impl`              | `class`                       |
|-------------|------------------------------|-------------------------------|
| Inspiration | Go, Rust                     | Java, C#, Kotlin              |
| Creation    | `Foo { field: val }`         | `new Foo(args)`               |
| Constructor | not needed                   | `init(params) { ... }`        |
| Methods     | in `impl Foo { ... }` block  | inside `class Foo { ... }`    |
| `self`      | explicit parameter           | explicit parameter            |

---

## 15. Project System and Imports

### Project Layout

```
myproject/
├── orbitron.toml       # project manifest
├── src/
│   ├── main.ot         # entry point (contains func main)
│   ├── math.ot         # module
│   └── geometry.ot     # another module
└── bin/                # output directory
```

### `orbitron.toml` Manifest

```toml
[project]
name    = "myproject"
version = "0.1.0"

[build]
main   = "src/main.ot"     # entry point
output = "bin/myproject"   # output binary path
```

### Importing Modules

```orbitron
// src/main.ot
import "math";       // loads src/math.ot
import "geometry";   // loads src/geometry.ot

func main() {
    println(add(2, 3));   // function from math.ot
}
```

Imports are resolved **before** code generation: the compiler traverses all files and merges their
ASTs. Circular imports result in a compile error. Each file is imported at most once.

### CLI Commands

```bash
# Create a new project
orbitron new myapp
cd myapp

# Build (searches for orbitron.toml up the directory tree)
orbitron build

# Build and run
orbitron run

# Flags
orbitron build -o bin/release      # override output path
orbitron build --emit-llvm         # save .ll file and stop
orbitron build --save-temps        # keep .ll and .s files
orbitron build -v                  # verbose pipeline output
```

### Single-file Mode

```bash
orbitron hello.ot                  # → ./hello
orbitron -o myapp hello.ot         # → ./myapp
orbitron --emit-llvm hello.ot      # → hello.ll (no linking)
orbitron -v examples/fib.ot        # verbose output
```

---

## 16. Standard Library

Orbitron ships with a set of standard library modules in the `stdlib/` folder.
They are imported with the `std/` prefix:

```orbitron
import "std/math";   // math functions
import "std/bits";   // bitwise operations
import "std/algo";   // utility algorithms
```

### `std/math`

| Function / constant  | Description |
|----------------------|-------------|
| `PI: float`          | π ≈ 3.14159... |
| `E: float`           | e ≈ 2.71828... |
| `INT_MAX: int`       | Maximum int value (i64) |
| `abs(x)`             | Absolute value |
| `max(a, b)`          | Maximum of two values |
| `min(a, b)`          | Minimum of two values |
| `clamp(val, lo, hi)` | Clamp val to [lo, hi] |
| `factorial(n)`       | n! (n >= 0) |
| `fib(n)`             | nth Fibonacci number (0-indexed) |
| `gcd(a, b)`          | Greatest common divisor |
| `lcm(a, b)`          | Least common multiple |
| `sum_range(a, b)`    | Sum of integers from a to b inclusive |
| `sign(x)`            | Sign: -1, 0, or 1 |
| `is_prime(n)`        | 1 if n is prime, else 0 |

```orbitron
import "std/math";

func main() {
    println(factorial(10));     // 3628800
    println(gcd(48, 18));       // 6
    println(is_prime(97));      // 1
    println(sum_range(1, 100)); // 5050
}
```

### `std/bits`

| Function              | Description |
|-----------------------|-------------|
| `bit_count(x)`        | Number of set bits (popcount) |
| `bit_len(x)`          | Bit length: floor(log2(x))+1 |
| `is_pow2(x)`          | 1 if x is a power of two |
| `next_pow2(x)`        | Next power of two >= x |
| `prev_pow2(x)`        | Previous power of two <= x |
| `low_bit(x)`          | Lowest set bit |
| `shl(x, n)`           | Left shift: x * 2^n |
| `shr(x, n)`           | Right shift: x / 2^n |
| `floor_log2(x)`       | Integer log2 (floor) |
| `reverse_bits(x, n)`  | Reverse the n lowest bits |

```orbitron
import "std/bits";

func main() {
    println(bit_count(255));   // 8
    println(is_pow2(16));      // 1
    println(next_pow2(5));     // 8
    println(shl(1, 10));       // 1024
    println(floor_log2(100));  // 6
}
```

### `std/algo`

| Function                         | Description |
|----------------------------------|-------------|
| `min3(a, b, c)`                  | Minimum of three |
| `max3(a, b, c)`                  | Maximum of three |
| `median3(a, b, c)`               | Median of three |
| `lerp(lo, hi, t)`                | Linear interpolation, t in [0..100] |
| `map_range(val, in_lo, in_hi, out_lo, out_hi)` | Remap value between ranges |
| `dist(a, b)`                     | Distance: \|a - b\| |
| `digit_count(x)`                 | Number of decimal digits |
| `digit_sum(x)`                   | Sum of digits |
| `reverse_digits(x)`              | Reverse decimal digits |
| `is_palindrome_num(x)`           | 1 if number is a palindrome |
| `ipow(base, exp)`                | Integer exponentiation (fast) |
| `triangle(n)`                    | Triangular number T(n) = n*(n+1)/2 |
| `is_triangle(n)`                 | 1 if n is a triangular number |
| `isqrt(n)`                       | Integer square root (floor) |
| `is_square(n)`                   | 1 if n is a perfect square |
| `near(a, b, tol)`                | 1 if \|a-b\| <= tol |
| `cycle(x, delta, n)`             | Cyclic offset: (x + delta) mod n |

```orbitron
import "std/algo";

func main() {
    println(ipow(2, 10));             // 1024
    println(isqrt(100));              // 10
    println(map_range(50, 0, 100, 0, 255)); // 127
    println(is_palindrome_num(121));  // 1
    println(cycle(6, 1, 7));          // 0
}
```

### stdlib Location

The `stdlib/` directory must be in one of these locations:
1. Next to the `orbitron` binary (recommended)
2. At `$ORBITRON_HOME/stdlib/`

### Adding Your Own Modules

Any `.ot` file inside a project's `src/` folder can be imported:

```orbitron
import "utils";      // loads src/utils.ot
import "net/http";   // loads src/net/http.ot
```

---

## 17. Compilation Backends

### LLVM (default)

Compiles to a native binary. Requires: `llc`, `clang`, `libm`.

```bash
orbitron build                    # LLVM backend (default)
orbitron hello.ot                 # single file → ./hello
orbitron build --emit-llvm        # stop at LLVM IR (.ll)
orbitron build --save-temps       # keep .ll and .s files
```

### JVM

Compiles to `.jar`. Requires: `javac`, `jar` (JDK).

```bash
orbitron build --backend jvm      # → bin/myapp.jar
orbitron run   --backend jvm      # build + run via java -jar
orbitron hello.ot --backend jvm   # single file → hello.jar
orbitron build --emit-java        # stop at Main.java
```

Run the compiled jar:
```bash
java -jar bin/myapp.jar
```

GraalVM native image:
```bash
native-image -jar bin/myapp.jar -o bin/myapp
```

### Backend Selection Priority

| Method                                | Priority |
|---------------------------------------|----------|
| `--backend llvm\|jvm` flag            | Highest  |
| `[build] backend = "jvm"` in `orbitron.toml` | Middle |
| Default: `llvm`                       | Lowest   |

---

## 18. Operator Precedence

From lowest to highest:

| Level | Operators                          | Associativity    |
|-------|------------------------------------|------------------|
| 1     | `\|>`                              | left             |
| 2     | `? :`                              | right            |
| 3     | `\|\|`                             | left             |
| 4     | `&&`                               | left             |
| 5     | `== != < <= > >=`                  | —                |
| 6     | `+ -`                              | left             |
| 7     | `* / %`                            | left             |
| 8     | `- !` (unary)                      | right            |
| 9     | `**`                               | right            |
| 10    | `.field` `.method()` `[idx]`       | left (postfix)   |

---

## 19. Grammar (EBNF, simplified)

```
program    = (func_decl | struct_decl | impl_decl | class_decl
           |  enum_decl | const_decl  | import_decl)* ;

import_decl = 'import' STRING ';' ;
func_decl   = 'func' IDENT '(' param_list ')' [':' type] block ;
const_decl  = 'const' IDENT [':' type] '=' expr ';' ;
enum_decl   = 'enum' IDENT '{' (IDENT ',')* '}' ;

block = '{' stmt* '}' ;
stmt  = var_stmt | const_stmt | assign | if_stmt | unless_stmt
      | while_stmt | do_while | for_stmt | repeat_stmt
      | loop_stmt | return_stmt | println_stmt | defer_stmt
      | match_stmt | field_assign | index_assign
      | compound_assign | expr ';' ;

expr      = pipe_expr ;
pipe_expr = ternary ('|>' IDENT ['(' arg_list ')'])* ;
ternary   = or_expr ['?' or_expr ':' ternary] ;
or_expr   = and_expr ('||' and_expr)* ;
and_expr  = cmp_expr ('&&' cmp_expr)* ;
cmp_expr  = add_expr [('=='|'!='|'<'|'<='|'>'|'>=') add_expr] ;
add_expr  = mul_expr (('+' | '-') mul_expr)* ;
mul_expr  = unary (('*' | '/' | '%') unary)* ;
unary     = ('-' | '!') unary | power ;
power     = postfix ['**' unary] ;
postfix   = call_base ('.' IDENT ['(' arg_list ')'] | '[' expr ']')* ;
call_base = 'new' IDENT '(' arg_list ')'
          | IDENT '(' arg_list ')'
          | IDENT '{' field_inits '}'
          | '[' arg_list ']'
          | 'readInt' '(' ')' | 'readFloat' '(' ')'
          | primary ;
primary   = INT | FLOAT | STRING | INTERP_STRING | IDENT | 'self'
          | 'true' | 'false' | '(' expr ')' ;
```

---

## Special Values

| Literal | Value     |
|---------|-----------|
| `true`  | `1` (int) |
| `false` | `0` (int) |

---

## Strings

String literals (`"..."`) are only allowed inside `println()`.
To embed variables, use `$"..."`.

```orbitron
println("Any text");
println("String with \"quotes\"");
println($"x = {x}");
```

---

## Comments

```orbitron
// Line comment

/* Block
   comment */
```
