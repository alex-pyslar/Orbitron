# Краткий справочник — Orbitron

Шпаргалка по всему синтаксису языка на одной странице.

---

## Переменные и константы

```orbitron
var x = 42;                 // переменная (тип выводится)
var pi: float = 3.14;       // с аннотацией типа
x = 100;                    // переприсвоение (без var)

const MAX: int   = 1000;    // константа
const TAX: float = 0.2;
```

---

## Типы

| Тип | Описание | Пример |
|-----|----------|--------|
| `int` | 64-бит знаковое целое | `0`, `42`, `-7` |
| `float` | 64-бит двойной точности | `3.14`, `-0.5` |
| `true` | Целое 1 | `var f = true;` |
| `false` | Целое 0 | `var d = false;` |

---

## Операторы

```orbitron
// Арифметика
x + y     x - y     x * y     x / y     x % y     x ** y

// Сравнение (возвращают 0 или 1)
x == y    x != y    x < y    x <= y    x > y    x >= y

// Логика
x && y    x || y    !x

// Составное присваивание
x += n    x -= n    x *= n    x /= n

// Тернарный
cond ? a : b

// Конвейер
x |> func
x |> func(extra_arg)
```

---

## Управление потоком

```orbitron
// if / else if / else
if (cond) { }
if (cond) { } else { }
if (cond) { } else if (cond2) { } else { }

// unless (выполняется если cond ЛОЖНО)
unless (cond) { }

// match
match expr {
    значение1 => { }
    Enum.Variant => { }
    _ => { }           // джокер
}

// Тернарный
var r = cond ? a : b;
```

---

## Циклы

```orbitron
for i in 0..10 { }          // [0, 10) — исключая 10
for i in 0..=10 { }         // [0, 10] — включая 10
for i in 0..m, j in 0..n { } // два диапазона одновременно

while (cond) { }
do { } while (cond);
loop { if (done) { break; } }
repeat 5 { }                 // ровно 5 раз

break;
continue;
```

---

## Функции

```orbitron
// Объявление
func name(a: int, b: int): int {
    return a + b;
}

// Без аннотаций
func name(a, b) {
    return a + b;
}

// Вызов
var r = name(3, 4);

// Конвейер
var r = 5 |> double |> inc;    // inc(double(5))
```

---

## Массивы

```orbitron
var a = [1, 2, 3, 4, 5];    // создание
var v = a[0];                // чтение (индекс с нуля)
a[2] = 99;                   // запись

for i in 0..5 { println(a[i]); }    // перебор
```

---

## Перечисления

```orbitron
enum Color { Red, Green, Blue }    // Red=0, Green=1, Blue=2

var c = Color.Red;                 // c == 0

match c {
    Color.Red   => { println("красный"); }
    Color.Green => { println("зелёный"); }
    _           => { println("другой"); }
}
```

---

## struct + impl (стиль Go/Rust)

```orbitron
struct Point {
    x: int,
    y: int,
}

impl Point {
    pub func len_sq(self): int {
        return self.x * self.x + self.y * self.y;
    }
    pub func move_by(self, dx: int, dy: int) {
        self.x += dx;
        self.y += dy;
    }
}

var p = Point { x: 3, y: 4 };   // без new!
println(p.len_sq());              // 25
p.move_by(1, 0);
```

---

## class + init (стиль Java/C#)

```orbitron
class Counter {
    private val: int,
    private step: int,

    init(start: int, s: int) {
        self.val  = start;
        self.step = s;
    }

    pub func tick(self) { self.val += self.step; }
    pub func get(self): int { return self.val; }
}

var c = new Counter(0, 5);
c.tick();
println(c.get());   // 5
```

---

## Строки и вывод

```orbitron
println("Обычная строка");
println($"Интерполяция: x={x}, pi={pi}");

// Только имена переменных в { }
// Только int и float
// Только внутри println()
```

---

## Defer

```orbitron
func example() {
    defer println("последний");    // выполняется в конце
    defer println("предпоследний");
    println("первый");
}
// вывод: первый → предпоследний → последний
```

---

## Проект

```toml
# orbitron.toml
[project]
name    = "myapp"
version = "0.1.0"

[build]
main    = "src/main.ot"
output  = "bin/myapp"
backend = "llvm"          # или "jvm"
```

```orbitron
// src/main.ot
import "utils";      // загружает src/utils.ot
import "std/math";   // загружает stdlib/math.ot

func main() { }
```

---

## Ввод

```orbitron
var n = readInt();     // прочитать int из stdin
var f = readFloat();   // прочитать float из stdin
```

---

## Низкоуровневые (только LLVM)

```orbitron
var addr = &x;           // адрес переменной
var val  = *addr;        // разыменование
ptr_write(addr, v);      // запись i64 по адресу
ptr_write_byte(addr, b); // запись байта по адресу
ptr_read(addr);          // чтение i64 по адресу
var p = cstr("hello");   // C-строка → адрес
var v = sign_ext(x);     // расширение знака (32→64 бит)

syscall(nr, a0, a1, ...);  // прямой системный вызов Linux

extern func name(a: int, ...): int;  // объявить C-функцию
```

---

## Стандартная библиотека

```orbitron
import "std/math";   // abs, max, min, clamp, factorial, fib, gcd, lcm,
                     // sum_range, sign, is_prime, PI, E, INT_MAX

import "std/bits";   // bit_count, bit_len, is_pow2, next_pow2, prev_pow2,
                     // low_bit, shl, shr, floor_log2, reverse_bits

import "std/algo";   // min3, max3, median3, lerp, map_range, dist, near,
                     // digit_count, digit_sum, reverse_digits, is_palindrome_num,
                     // ipow, isqrt, is_square, triangle, is_triangle, cycle

import "std/sys";    // SYS_*, STDIN, STDOUT, STDERR,
                     // sys_alloc, sys_free, sys_write, sys_read,
                     // sys_exit, sys_getpid, sys_sleep, ...

import "std/net";    // tcp_socket, udp_socket, net_ip, tcp_connect,
                     // net_bind, net_listen, net_accept, net_send,
                     // net_recv, net_close, net_reuseaddr

import "std/db";     // db_open, db_close, db_exec, db_prepare,
                     // db_step, db_finalize, db_col_int, db_col_count,
                     // SQLITE_OK, SQLITE_ROW, SQLITE_DONE
```

---

## CLI

```bash
orbitron new <name>                  # создать проект
orbitron build                       # собрать проект
orbitron run                         # собрать + запустить
orbitron <file.ot>                   # скомпилировать один файл

# Флаги
-o <file>                            # имя выходного файла
--backend llvm|jvm                   # бэкенд
--emit-llvm                          # сохранить .ll
--emit-java                          # сохранить .java
--save-temps                         # сохранить .ll и .s
-v, --verbose                        # подробный вывод
```

---

## Приоритет операторов

| Уровень | Операторы | Ассоциативность |
|---------|-----------|----------------|
| 1 (низший) | `\|>` | левая |
| 2 | `? :` | правая |
| 3 | `\|\|` | левая |
| 4 | `&&` | левая |
| 5 | `== != < <= > >=` | — |
| 6 | `+ -` | левая |
| 7 | `* / %` | левая |
| 8 | `-` `!` (унарные) | правая |
| 9 | `**` | правая |
| 10 (высший) | `.поле` `.метод()` `[индекс]` | левая |

---

## Комментарии

```orbitron
// Однострочный комментарий
/* Многострочный
   комментарий */
```

---

## Специальные значения

| Литерал | Значение |
|---------|---------|
| `true`  | `1` (int) |
| `false` | `0` (int) |

---

## Грамматика (EBNF, упрощённая)

```ebnf
program    = ( func_decl | struct_decl | impl_decl | class_decl
             | enum_decl  | const_decl  | import_decl )* ;

import_decl = 'import' STRING ';' ;
const_decl  = 'const' IDENT ':' type '=' expr ';' ;
enum_decl   = 'enum'  IDENT '{' (IDENT ',')* '}' ;
func_decl   = 'func'  IDENT '(' param_list ')' [':' type] block ;

block = '{' stmt* '}' ;
stmt  = var_stmt | assign | compound_assign | if_stmt | unless_stmt
      | while_stmt | do_while | for_stmt | repeat_stmt | loop_stmt
      | return_stmt | println_stmt | defer_stmt | match_stmt
      | field_assign | index_assign | expr ';' ;

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
          | IDENT '{' field_inits '}'
          | IDENT '(' arg_list ')'
          | '[' arg_list ']'
          | '&' IDENT | '*' IDENT
          | 'readInt' '(' ')' | 'readFloat' '(' ')'
          | primary ;
primary   = INT | FLOAT | STRING | INTERP_STRING | IDENT | 'self'
          | 'true' | 'false' | '(' expr ')' ;
```
