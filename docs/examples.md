# Orbitron Examples

All examples are available in the `examples/` directory.

## Example Index

| Path                          | Mode           | Demonstrates |
|-------------------------------|----------------|--------------|
| `examples/hello.ot`           | Single file    | Basic output, variables, arithmetic |
| `examples/fibonacci.ot`       | Single file    | Recursion, while loop, for..in |
| `examples/oop_struct.ot`      | Single file    | struct + impl, multi-range for |
| `examples/oop_class.ot`       | Single file    | class + init, encapsulation |
| `examples/features.ot`        | Single file    | **All 10 new features** — const, **, `\|>`, unless, $"", [], enum, defer, repeat, ternary |
| `examples/stats.ot`           | Single file    | **Combined**: struct+impl, class, all 10 features together |
| `examples/net_demo.ot`        | Single file    | TCP sockets, syscalls, extern func |
| `examples/calculator/`        | Project (multi)| Build system, import, multi-file structure |
| `examples/geometry/`          | Project (multi)| Two-module import, struct+impl, enum, all features |

---

## Running Examples

```bash
# Single file
orbitron examples/hello.ot && ./hello
orbitron -o stats examples/stats.ot && ./stats

# Project (cd into the directory first)
cd examples/geometry
orbitron run

cd examples/calculator
orbitron build && ./bin/calculator
```

---

## 1. Hello World (`examples/hello.ot`)

The minimal program: print a string and do basic arithmetic.

```orbitron
func main() {
    println("Hello, World!");

    var x = 42;
    println(x);          // 42

    var pi: float = 3.14159;
    println(pi);         // 3.14159

    var sum = x + 8;
    println(sum);        // 50
}
```

**Key concepts:** `println`, `var`, optional type annotations.

---

## 2. Fibonacci Numbers (`examples/fibonacci.ot`)

Recursion and iteration — two approaches to the same problem.

```orbitron
func fib_rec(n: int): int {
    if (n <= 1) { return n; }
    return fib_rec(n - 1) + fib_rec(n - 2);
}

func fib_iter(n: int): int {
    if (n <= 1) { return n; }
    var a = 0;
    var b = 1;
    var i = 2;
    while (i <= n) {
        var tmp = a + b;
        a = b;
        b = tmp;
        i += 1;
    }
    return b;
}

func main() {
    for i in 0..=15 {
        println(fib_iter(i));
    }
    // 0 1 1 2 3 5 8 13 21 34 55 89 144 233 377 610
}
```

**Key concepts:** recursion, `while`, `for i in 0..=N` (inclusive).

---

## 3. Structs — Go/Rust Style (`examples/oop_struct.ot`)

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

func main() {
    var p = Point { x: 3, y: 4 };  // struct literal — no new
    println(p.dist_sq());   // 25

    p.move_by(1, -1);
    println(p.x);   // 4

    // 3×3 table of squared distances
    for i in 0..3, j in 0..3 {
        var pt = Point { x: i, y: j };
        println(pt.dist_sq());
    }
    // 0 1 4 1 2 5 4 5 8
}
```

**Key concepts:** `struct`, `impl`, struct literal `Name { field: val }`, `for i in ..., j in ...`.

---

## 4. Classes — Java/C# Style (`examples/oop_class.ot`)

```orbitron
class BankAccount {
    private balance: int,

    init(initial: int) {
        self.balance = initial;
    }

    pub func deposit(self, amount: int) {
        if (amount > 0) {
            self.balance = self.balance + amount;
        }
    }

    pub func withdraw(self, amount: int): int {
        if (amount > 0 && self.balance >= amount) {
            self.balance = self.balance - amount;
            return 1;   // success
        }
        return 0;       // insufficient funds
    }

    pub func get_balance(self): int {
        return self.balance;
    }
}

func main() {
    var acc = new BankAccount(500);
    acc.deposit(200);
    println(acc.get_balance());   // 700

    var ok = acc.withdraw(300);
    println(ok);                  // 1
    println(acc.get_balance());   // 400

    println(acc.withdraw(1000));  // 0 (insufficient funds)
}
```

**Key concepts:** `class`, `private`, `init`, `new ClassName(args)`.

---

## 5. All 10 New Features (`examples/features.ot`)

Each of the 10 language features shown in a standalone block.

### 1. `const` — Constants *(Rust / C++)*
```orbitron
const PI: int       = 3;
const MAX_SIZE: int = 10;

func main() {
    var area = PI * 5 * 5;   // 75
    println(area);
}
```

### 2. `**` — Exponentiation *(Python)*
```orbitron
var p = 2 ** 10;    // 1024
var q = 3 ** 4;     //   81
```

### 3. `|>` — Pipe Operator *(Elixir / F#)*
```orbitron
func double(n: int): int { return n * 2; }
func inc(n: int):    int { return n + 1; }
func square(n: int): int { return n * n; }

var result = 3 |> double |> inc |> square;  // 49
// equivalent to: square(inc(double(3)))
```

### 4. `unless` — Inverted Conditional *(Ruby)*
```orbitron
var x = 0;
unless (x != 0) {
    println(42);   // executes because x == 0
}
```

### 5. `$"..."` — String Interpolation *(C# / Kotlin)*
```orbitron
var score = 100;
println($"score: {score}");    // score: 100
println($"2^10 = {p}");        // 2^10 = 1024
```

### 6. `[...]` — Arrays *(Python / JS)*
```orbitron
var primes = [2, 3, 5, 7, 11, 13];
println(primes[4]);   // 11
primes[0] = 99;       // mutation
var sum = 0;
for i in 0..6 { sum += primes[i]; }
```

### 7. `enum` — Enumerations *(Rust / Swift)*
```orbitron
enum Season { Spring, Summer, Autumn, Winter }
var s = Season.Summer;   // s == 1

match s {
    Season.Spring => { println("Spring!"); }
    Season.Summer => { println("Summer!"); }
    _             => { println("Other"); }
}
```

### 8. `defer` — Deferred Execution *(Go)*
```orbitron
func run() {
    defer println("Bye!");   // runs last
    println("Hello");
    println("World");
    // → Hello, World, Bye!
}
```

### 9. `repeat N` — Repeat N Times *(Lua / Pascal)*
```orbitron
repeat 5 {
    println("Hi!");   // 5 times
}
var counter = 0;
repeat 10 { counter += 1; }
// counter == 10
```

### 10. `? :` — Ternary Operator *(C / Java)*
```orbitron
var max   = a > b ? a : b;
var label = n > 10 ? 3 : n > 0 ? 2 : 1;   // chained
```

---

## 6. Data Analysis (`examples/stats.ot`)

A comprehensive example combining **all** language features in a single program.

```orbitron
const N:     int = 6;
const LIMIT: int = 100;

enum Grade { Fail, Pass, Good, Excellent }

struct Summary {
    lo: int, hi: int, total: int,
}

impl Summary {
    pub func spread(self): int    { return self.hi - self.lo; }
    pub func avg(self): int       { return self.total / N; }
    pub func spread_sq(self): int { return self.spread() ** 2; }
}

class Tracker {
    private total: int, private cnt: int,
    private lo: int,    private hi:  int,

    init(first: int) {
        self.total = first; self.cnt = 1;
        self.lo = first;    self.hi  = first;
    }

    pub func push(self, v: int) {
        self.total = self.total + v;
        self.cnt   = self.cnt + 1;
        unless (v >= self.lo) { self.lo = v; }
        if (v > self.hi)      { self.hi = v; }
    }

    pub func sum(self):  int { return self.total; }
    pub func mean(self): int { return self.total / self.cnt; }
    pub func min(self):  int { return self.lo; }
    pub func max(self):  int { return self.hi; }
}

func double(n: int): int { return n * 2; }
func inc(n: int):    int { return n + 1; }

func classify(score: int): int {
    return score >= 90 ? Grade.Excellent :
           score >= 70 ? Grade.Good      :
           score >= 50 ? Grade.Pass      : Grade.Fail;
}

func main() {
    defer println("=== Analysis complete ===");

    var data = [55, 78, 92, 67, 85, 61];
    var tr = new Tracker(data[0]);
    for i in 1..N { tr.push(data[i]); }

    var s = Summary { lo: tr.min(), hi: tr.max(), total: tr.sum() };
    var avg = s.avg();
    var sq  = s.spread_sq();

    println($"avg={avg}");
    println($"spread^2={sq}");

    var bonus = tr.mean() |> double |> inc;
    println($"bonus={bonus}");

    for i in 0..N {
        match classify(data[i]) {
            Grade.Excellent => { println("Excellent"); }
            Grade.Good      => { println("Good"); }
            Grade.Pass      => { println("Pass"); }
            Grade.Fail      => { println("Fail"); }
            _               => {}
        }
    }

    var warmup = 0;
    repeat 5 { warmup += 1; }
    println($"warmup={warmup}");

    unless (avg >= 90) { println("Below excellent average"); }

    println($"2^8={2 ** 8}");
}
```

Build and run:
```bash
orbitron -o stats examples/stats.ot && ./stats
```

---

## 7. Calculator — Multi-file Project (`examples/calculator/`)

Demonstrates the build system: `orbitron.toml` + `import`.

### Layout
```
examples/calculator/
├── orbitron.toml
├── src/
│   ├── main.ot      # import "math"
│   └── math.ot      # add, sub, mul, pow2
└── bin/
    └── calculator   # output binary
```

### `orbitron.toml`
```toml
[project]
name = "calculator"
version = "0.1.0"

[build]
main   = "src/main.ot"
output = "bin/calculator"
```

### `src/math.ot`
```orbitron
func add(a: int, b: int): int { return a + b; }
func sub(a: int, b: int): int { return a - b; }
func mul(a: int, b: int): int { return a * b; }
func pow2(n: int): int        { return n ** 2; }
```

### `src/main.ot`
```orbitron
import "math";

func main() {
    var a = 10;
    var b = 3;
    println(add(a, b));   // 13
    println(sub(a, b));   //  7
    println(mul(a, b));   // 30
    println(pow2(a));     // 100
}
```

Build:
```bash
cd examples/calculator
orbitron build     # → bin/calculator
orbitron run       # build + run
```

---

## 8. Geometry — Multi-file Project (`examples/geometry/`)

Two modules (`vectors`, `shapes`) plus all language features.

### Layout
```
examples/geometry/
├── orbitron.toml
├── src/
│   ├── main.ot      # import "vectors"; import "shapes"
│   ├── vectors.ot   # struct Vec2 + impl
│   └── shapes.ot    # rect_area, circle_area, hyp_sq, ...
└── bin/
```

### `src/vectors.ot` (excerpt)
```orbitron
const ORIGIN_X: int = 0;
const ORIGIN_Y: int = 0;

struct Vec2 { x: int, y: int }

impl Vec2 {
    pub func len_sq(self): int    { return self.x ** 2 + self.y ** 2; }
    pub func manhattan(self): int {
        var ax = self.x > 0 ? self.x : -self.x;
        var ay = self.y > 0 ? self.y : -self.y;
        return ax + ay;
    }
    pub func scale(self, f: int) { self.x = self.x * f; self.y = self.y * f; }
    pub func get_x(self): int    { return self.x; }
    pub func get_y(self): int    { return self.y; }
}
```

### `src/shapes.ot` (excerpt)
```orbitron
const PI: int = 3;

func rect_area(w: int, h: int): int  { return w * h; }
func circle_area(r: int): int        { return PI * r * r; }
func hyp_sq(a: int, b: int): int     { return a ** 2 + b ** 2; }

// 0=scalene, 1=isosceles, 2=equilateral
func triangle_type(a: int, b: int, c: int): int {
    return a == b && b == c ? 2 :
           a == b || b == c || a == c ? 1 : 0;
}
```

### `src/main.ot` (excerpt)
```orbitron
import "vectors";
import "shapes";

enum TriType  { Scalene, Isosceles, Equilateral }
enum Quadrant { Q1, Q2, Q3, Q4, Origin }

func double(n: int): int { return n * 2; }
func inc(n: int):    int { return n + 1; }

func main() {
    defer println("=== Geometry done ===");

    var v1 = Vec2 { x: 3, y: 4 };
    println($"len_sq={v1.len_sq()}");       // 25
    println($"manhattan={v1.manhattan()}"); // 7

    v1.scale(2);
    println($"scaled=({v1.get_x()},{v1.get_y()})");  // (6,8)

    // pipe operator
    var piped = v1.len_sq() |> double |> inc;
    println($"piped={piped}");

    // shapes
    println($"rect_area={rect_area(5, 4)}");    // 20
    println($"circle_area={circle_area(3)}");   // 27

    // enum + match
    var tt = triangle_type(3, 3, 3);
    match tt {
        TriType.Equilateral => { println("equilateral"); }
        TriType.Isosceles   => { println("isosceles"); }
        TriType.Scalene     => { println("scalene"); }
        _                   => {}
    }

    // multi-range for: 3×3 grid
    for i in 0..3, j in 0..3 {
        var pt = Vec2 { x: i, y: j };
        println(pt.len_sq());
    }

    var steps = 0;
    repeat 8 { steps += 1; }
    println($"steps={steps}");
}
```

Build:
```bash
cd examples/geometry
orbitron run
```

---

## When to Choose Each OOP Style

| Situation                                    | Recommendation  |
|----------------------------------------------|-----------------|
| Data + computations, no mutable state        | `struct + impl` |
| Encapsulated mutable state                   | `class + init`  |
| Geometry, physics, math                      | `struct + impl` |
| Counter, queue, bank account                 | `class + init`  |

Both styles generate identical LLVM IR — the difference is purely syntactic.
