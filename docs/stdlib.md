# Orbitron Standard Library

The standard library contains ready-to-use functions and constants written in Orbitron itself.
All stdlib modules are available via `import "std/<module>"`.

## Installation

The `stdlib/` directory must be in one of these locations (checked in order):

1. `$ORBITRON_HOME/stdlib/` — if the environment variable is set
2. `{exe_dir}/stdlib/` — next to the `orbitron` binary

When building from source:
```bash
cargo build --release
# stdlib/ already lives in the project root — just run orbitron from there
```

---

## Modules

### `std/math` — Mathematics

```orbitron
import "std/math";
```

#### Constants

| Constant  | Type    | Value                              |
|-----------|---------|------------------------------------|
| `PI`      | `float` | 3.14159265358979 (π)               |
| `E`       | `float` | 2.71828182845905 (e)               |
| `INT_MAX` | `int`   | 9223372036854775807 (max i64)      |

#### Functions

| Signature                                | Description                                         |
|------------------------------------------|-----------------------------------------------------|
| `abs(x: int): int`                       | Absolute value \|x\|                               |
| `max(a: int, b: int): int`               | Maximum of two numbers                             |
| `min(a: int, b: int): int`               | Minimum of two numbers                             |
| `clamp(val: int, lo: int, hi: int): int` | Clamp val to the range [lo, hi]                    |
| `factorial(n: int): int`                 | n! — factorial (n >= 0)                            |
| `fib(n: int): int`                       | nth Fibonacci number (fib(0)=0, fib(1)=1)          |
| `gcd(a: int, b: int): int`               | Greatest common divisor (Euclidean algorithm)      |
| `lcm(a: int, b: int): int`               | Least common multiple                              |
| `sum_range(a: int, b: int): int`         | Sum of integers from a to b inclusive              |
| `sign(x: int): int`                      | Sign of x: -1, 0, or 1                            |
| `is_prime(n: int): int`                  | 1 if n is prime, else 0 (n > 1)                   |

#### Examples

```orbitron
import "std/math";

func main() {
    println(abs(-7));             // 7
    println(max(10, 20));         // 20
    println(clamp(150, 0, 100));  // 100
    println(factorial(10));       // 3628800
    println(fib(10));             // 55
    println(gcd(12, 18));         // 6
    println(lcm(4, 6));           // 12
    println(sum_range(1, 100));   // 5050
    println(sign(-42));           // -1
    println(is_prime(97));        // 1
    println(is_prime(100));       // 0
}
```

---

### `std/bits` — Bitwise Operations

```orbitron
import "std/bits";
```

All functions operate on integer values (`int` = i64).
Bit operations are implemented via arithmetic (not native shift/AND instructions),
so they are compatible with both backends (LLVM and JVM).

#### Functions

| Signature                              | Description |
|----------------------------------------|-------------|
| `bit_count(x: int): int`              | Number of set bits (popcount) |
| `bit_len(x: int): int`                | Bit length: `floor(log2(x)) + 1` for x>0; 0 for x<=0 |
| `is_pow2(x: int): int`                | 1 if x is a power of two (x > 0) |
| `next_pow2(x: int): int`              | Next power of two >= x (x >= 1) |
| `prev_pow2(x: int): int`              | Previous power of two <= x (x >= 1) |
| `low_bit(x: int): int`                | Lowest set bit; 0 if x == 0 |
| `shl(x: int, n: int): int`            | Left shift: x * 2^n |
| `shr(x: int, n: int): int`            | Right shift: x / 2^n |
| `floor_log2(x: int): int`             | Integer log2 (floor): times x can be halved to reach 1 |
| `reverse_bits(x: int, bits: int): int`| Reverse the `bits` lowest bits of x |

#### Examples

```orbitron
import "std/bits";

func main() {
    // popcount
    println(bit_count(255));    // 8  (0b11111111)
    println(bit_count(7));      // 3  (0b111)

    // bit length
    println(bit_len(256));      // 9  (0b100000000)
    println(bit_len(255));      // 8

    // powers of two
    println(is_pow2(8));        // 1
    println(is_pow2(9));        // 0
    println(next_pow2(5));      // 8
    println(prev_pow2(9));      // 8

    // shifts
    println(shl(1, 10));        // 1024
    println(shr(1024, 3));      // 128

    // log2
    println(floor_log2(100));   // 6  (2^6=64 <= 100 < 128=2^7)

    // bit reversal
    println(reverse_bits(11, 4)); // 13  (0b1011 -> 0b1101)
}
```

#### Practical: align to power of two

```orbitron
import "std/bits";

func main() {
    var size    = 300;
    var aligned = next_pow2(size);
    println(aligned);   // 512
}
```

---

### `std/algo` — Algorithms

```orbitron
import "std/algo";
```

#### Functions

##### Three-value comparisons

| Signature                              | Description |
|----------------------------------------|-------------|
| `min3(a: int, b: int, c: int): int`    | Minimum of three |
| `max3(a: int, b: int, c: int): int`    | Maximum of three |
| `median3(a: int, b: int, c: int): int` | Median of three |

##### Interpolation and range mapping

| Signature | Description |
|-----------|-------------|
| `lerp(lo: int, hi: int, t: int): int` | Linear interpolation: lo + t*(hi-lo)/100; t in [0..100] |
| `map_range(val, in_lo, in_hi, out_lo, out_hi: int): int` | Remap a value from one range to another |

##### Distances

| Signature                     | Description |
|-------------------------------|-------------|
| `dist(a: int, b: int): int`   | Distance: \|a - b\| |

##### Digit operations

| Signature                        | Description |
|----------------------------------|-------------|
| `digit_count(x: int): int`       | Number of digits in \|x\| (min 1) |
| `digit_sum(x: int): int`         | Sum of digits of \|x\| |
| `reverse_digits(x: int): int`    | Reverse digits: reverse_digits(1234) = 4321 |
| `is_palindrome_num(x: int): int` | 1 if the number is a palindrome |

##### Powers and sequences

| Signature                        | Description |
|----------------------------------|-------------|
| `ipow(base: int, exp: int): int` | Fast integer exponentiation |
| `triangle(n: int): int`          | Triangular number T(n) = n*(n+1)/2 |
| `is_triangle(n: int): int`       | 1 if n is a triangular number |
| `isqrt(n: int): int`             | Integer square root: floor(sqrt(n)) |
| `is_square(n: int): int`         | 1 if n is a perfect square |

##### Miscellaneous

| Signature                                | Description |
|------------------------------------------|-------------|
| `near(a: int, b: int, tol: int): int`    | 1 if \|a-b\| <= tol |
| `cycle(x: int, delta: int, n: int): int` | Cyclic offset (x+delta) mod n, result >= 0 |

#### Examples

```orbitron
import "std/algo";

func main() {
    // three-value comparisons
    println(min3(7, 2, 9));      // 2
    println(max3(7, 2, 9));      // 9
    println(median3(7, 2, 9));   // 7

    // interpolation: 75% of [0..255] → 191
    println(lerp(0, 255, 75));   // 191

    // ADC [0..1023] → brightness [0..255]
    println(map_range(512, 0, 1023, 0, 255)); // 127

    // digit operations
    println(digit_sum(1234));           // 10
    println(reverse_digits(1234));      // 4321
    println(is_palindrome_num(12321));  // 1

    // powers
    println(ipow(2, 16));   // 65536
    println(isqrt(144));    // 12
    println(is_square(49)); // 1

    // triangular numbers
    println(triangle(5));     // 15
    println(is_triangle(15)); // 1

    // cyclic day counter (0–6)
    var day = 5;
    println(cycle(day, 3, 7)); // 1  (5+3 = 8 mod 7 = 1)
}
```

---

## Combining Modules

Multiple modules can be imported at once:

```orbitron
import "std/math";
import "std/bits";
import "std/algo";

func main() {
    // Find the nearest power of two to the factorial of 5
    var f = factorial(5);   // 120  (from std/math)
    var p = next_pow2(f);   // 128  (from std/bits)
    var s = isqrt(p);       // 11   (from std/algo)
    println(s);
}
```

---

## Writing Your Own stdlib Modules

Create a file `stdlib/mymodule.ot` with ordinary Orbitron code:

```orbitron
// stdlib/mymodule.ot
const MY_CONST: int = 42;

func my_func(x: int): int {
    return x * MY_CONST;
}
```

Import it in your project:

```orbitron
import "std/mymodule";

func main() {
    println(my_func(2));  // 84
}
```

---

## Current Limitations

- Arrays cannot be passed to stdlib functions (all parameters are `int` or `float`).
- Strings are only usable inside `println()` — no string functions in stdlib.
- All constants and functions are in the global namespace (name conflicts can occur
  if multiple modules define the same identifiers).
