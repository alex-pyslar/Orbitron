# Краткий справочник — Orbitron

Шпаргалка по всему синтаксису языка на одной странице.

---

## Переменные и константы

```orbitron
let x = 42;                 // иммутабельная переменная (тип выводится)
let pi: f64 = 3.14;         // с аннотацией типа
mut n = 0;                  // мутабельная переменная
n = 100;                    // переприсвоение (только mut)

#const MAX: i64 = 1000;     // константа
#const TAX: f64 = 0.2;
```

> Ключевые слова `var`, `func`, `const`, `import` сохраняются для совместимости.
> Новый код должен использовать `let`/`mut`, `fn`, `#const`, `#import`.

---

## Типы

| Тип | Псевдоним | Описание | Пример |
|-----|-----------|----------|--------|
| `i64` | `int` | 64-бит знаковое целое | `0`, `42`, `-7` |
| `f64` | `float` | 64-бит двойной точности | `3.14`, `-0.5` |
| `true` | — | Целое 1 | `let f = true;` |
| `false` | — | Целое 0 | `let d = false;` |

---

## Операторы

```orbitron
// Арифметика
x + y     x - y     x * y     x / y     x % y     x ** y

// Сравнение (возвращают 0 или 1)
x == y    x != y    x < y    x <= y    x > y    x >= y

// Логика
x && y    x || y    !x

// Побитовые
x ^ y     ~x

// Составное присваивание
x += n    x -= n    x *= n    x /= n    x %= n    x ^= n

// Тернарный
cond ? a : b

// Конвейер
x |> func
x |> func(extra_arg)
```

---

## Кортежи

```orbitron
let (a, b) = (1, 2);        // деструктурирующее присвоение
let (x, y) = (y, x);        // обмен значений
let point  = (10, 20);      // кортеж как значение
```

---

## Аннотации

```orbitron
@test
fn test_something() { assert_eq!(1 + 1, 2); }

@deprecated
fn old_api(x: i64): i64 { return x; }

@override
public fn speak(self) { println!("Гав!"); }

@inline
fn fast_mul(a: i64, b: i64): i64 => a * b;
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

// match (как инструкция)
match expr {
    значение1 => { }
    Enum.Variant => { }
    _ => { }           // джокер
}

// match как выражение (возвращает значение)
var v = match expr {
    1 => 10
    2 => 20
    _ => 0
};

// Тернарный
var r = cond ? a : b;

// assert! / assert_eq!
assert!(cond);
assert_eq!(a, b);
```

---

## Циклы

```orbitron
for i in 0..10 { }          // [0, 10) — исключая 10
for i in 0..=10 { }         // [0, 10] — включая 10
for i in 0..m, j in 0..n { } // два диапазона одновременно
for x in arr { }             // прямой перебор массива (как Python)

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
fn name(a: i64, b: i64): i64 {
    return a + b;
}

// Fat-arrow тело
fn name(a: i64, b: i64): i64 => a + b;

// Аннотация возврата через ->
fn name(a: i64, b: i64) -> i64 {
    return a + b;
}

// Параметры по умолчанию
fn greet(times: i64, gap: i64 = 1) { }

// Без аннотаций
fn name(a, b) {
    return a + b;
}

// Вызов
let r = name(3, 4);

// Конвейер
let r = 5 |> double |> inc;    // inc(double(5))

// Лямбда (анонимная функция)
let double = |x| x * 2;
let add    = |a, b| a + b;
let result = |x| { let y = x * 2; return y + 1; };

// Статический метод
pub static fn create(): i64 => 0;

// Вызов статического метода
let v = Type::create();
```

---

## Массивы

```orbitron
var a = [1, 2, 3, 4, 5];    // создание
var v = a[0];                // чтение (индекс с нуля)
a[2] = 99;                   // запись

for i in 0..5 { println(a[i]); }    // перебор по индексу
for x in a   { println(x); }        // прямой перебор
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

// match как выражение с enum
var name = match c {
    Color.Red   => 1
    Color.Green => 2
    Color.Blue  => 3
    _           => 0
};
```

---

## struct + impl (стиль Go/Rust)

```orbitron
struct Point {
    x: i64,
    y: i64,
}

impl Point {
    public fn len_sq(self): i64 => self.x * self.x + self.y * self.y;
    public fn move_by(self, dx: i64, dy: i64) {
        self.x += dx;
        self.y += dy;
    }
    pub static fn origin(): Point => Point { x: 0, y: 0 };   // статический метод
}

let p = Point { x: 3, y: 4 };   // без new!
println!(p.len_sq());             // 25
p.move_by(1, 0);
let o = Point::origin();         // статический вызов
```

---

## class + init (стиль Java/C#)

```orbitron
class Counter {
    private val: i64,
    private step: i64,

    init(start: i64, s: i64) {
        self.val  = start;
        self.step = s;
    }

    public fn tick(self) { self.val += self.step; }
    public fn get(self): i64 => self.val;

    pub static fn zero(): i64 => 0;   // статический метод
}

let c = new Counter(0, 5);
c.tick();
println!(c.get());           // 5
let z = Counter::zero();    // статический вызов
```

---

## Наследование классов

```orbitron
class Animal {
    private name: i64,
    init(n: i64) { self.name = n; }
    public fn get_name(self): i64 => self.name;
    public fn speak(self) { println!("..."); }
}

class Dog extends Animal {
    init(n: i64) { self.name = n; }

    @override
    public fn speak(self) { println!("Гав!"); }
}

let d = new Dog(1);
d.speak();   // Гав!
```

---

## Трейты

```orbitron
// Объявление трейта
trait Printable {
    fn print_info(self);
}

trait Measurable {
    fn area(self): i64;
    fn perimeter(self): i64;
}

// Реализация трейта для типа
struct Circle { radius: i64 }

impl Measurable for Circle {
    public fn area(self): i64 => 314 * self.radius * self.radius / 100;
    public fn perimeter(self): i64 => 628 * self.radius / 100;
}

// Реализация операторного трейта
impl Add for Circle {
    public fn add(self, other_r: i64): i64 => self.radius + other_r;
}
```

---

## Строки и вывод

```orbitron
println!("Обычная строка");
println!("Интерполяция: x=\{x}, pi=\{pi}");

// Только имена переменных в \{ }
// Только i64 и f64
// Только внутри println!()
```

---

## Defer

```orbitron
fn example() {
    defer println!("последний");    // выполняется в конце
    defer println!("предпоследний");
    println!("первый");
}
// вывод: первый → предпоследний → последний
```

---

## Assert

```orbitron
assert!(x > 0);           // аварийное завершение если x <= 0
assert_eq!(a, b);         // аварийное завершение если a != b
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
#import "utils";      // загружает src/utils.ot
#import "std/math";   // загружает stdlib/math.ot

fn main() { }
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
let val  = *addr;        // разыменование
ptr_write(addr, v);      // запись i64 по адресу
ptr_write_byte(addr, b); // запись байта по адресу
ptr_read(addr);          // чтение i64 по адресу
let p = cstr("hello");   // C-строка → адрес
let v = sign_ext(x);     // расширение знака (32→64 бит)

syscall(nr, a0, a1, ...);  // прямой системный вызов Linux

extern fn name(a: i64, ...): i64;   // объявить C-функцию
```

---

## Стандартная библиотека

```orbitron
#import "std/math";   // abs, max, min, clamp, factorial, fib, gcd, lcm,
                      // sum_range, sign, is_prime, PI, E, INT_MAX

#import "std/bits";   // bit_count, bit_len, is_pow2, next_pow2, prev_pow2,
                      // low_bit, shl, shr, floor_log2, reverse_bits

#import "std/algo";   // min3, max3, median3, lerp, map_range, dist, near,
                      // digit_count, digit_sum, reverse_digits, is_palindrome_num,
                      // ipow, isqrt, is_square, triangle, is_triangle, cycle

#import "std/sys";    // SYS_*, STDIN, STDOUT, STDERR,
                      // sys_alloc, sys_free, sys_write, sys_read,
                      // sys_exit, sys_getpid, sys_sleep, ...

#import "std/net";    // tcp_socket, udp_socket, net_ip, tcp_connect,
                      // net_bind, net_listen, net_accept, net_send,
                      // net_recv, net_close, net_reuseaddr

#import "std/db";     // db_open, db_close, db_exec, db_prepare,
                      // db_step, db_finalize, db_col_int, db_col_count,
                      // SQLITE_OK, SQLITE_ROW, SQLITE_DONE
```

---

## CLI

```bash
orbitron new <name>                  # создать проект
orbitron build                       # собрать проект
orbitron run                         # собрать + запустить
orbitron fmt                         # форматировать (вывод в stdout)
orbitron fmt --write                 # форматировать (записать в файл)
orbitron fmt --write src/main.ot     # форматировать конкретный файл
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
| 8 | `-` `!` `~` (унарные) | правая |
| 9 | `**` | правая |
| 10 (высший) | `.поле` `.метод()` `[индекс]` `::` | левая |

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

## Новый синтаксис (v2)

| Конструкция | Пример | Описание |
|-------------|--------|---------|
| Иммутабельная переменная | `let x = 5` | Заменяет `var` для неизменяемых |
| Мутабельная переменная | `mut x = 5` | Заменяет `var` для изменяемых |
| Функция | `fn f(x: i64): i64 { }` | Заменяет `func` |
| Fat-arrow тело | `fn f(x: i64): i64 => x * 2;` | Однострочная функция |
| Константа | `#const N: i64 = 5` | Заменяет `const` |
| Импорт | `#import "std/math"` | Заменяет `import` |
| Макрос печати | `println!("...")` | Заменяет `println(...)` |
| Интерполяция | `"val=\{x}"` | Заменяет `$"val={x}"` |
| Elvis оператор | `val ?: default` | Если val==0, вернуть default |
| Типы i64/f64 | `let x: i64 = 0` | Заменяет `int`/`float` |
| Побитовое NOT | `~x` | Инвертирует все биты `x` |
| Побитовое XOR | `a ^ b` | Исключающее ИЛИ побитово |
| XOR присваивание | `x ^= mask` | `x = x ^ mask` |
| Лямбда | `\|x\| x * 2` | Анонимная функция |
| Статический вызов | `Type::method()` | Вызов статического метода |
| Кортеж | `let (a, b) = (1, 2)` | Деструктурирование |
| Аннотация | `@test`, `@override` | Метаданные |
| Наследование | `class Dog extends Animal` | Дочерний класс |
| Трейт | `trait Foo { fn bar(self); }` | Интерфейс |
| impl трейта | `impl Foo for Bar { ... }` | Реализация трейта |
| for по массиву | `for x in arr { }` | Прямой перебор |
| match-выражение | `let v = match x { 1 => 10 _ => 0 };` | match возвращает значение |
| assert! | `assert!(x > 0)` | Проверка условия |
| assert_eq! | `assert_eq!(a, b)` | Проверка равенства |
| Форматировщик | `orbitron fmt --write` | Форматирование кода |

---

## Грамматика (EBNF, упрощённая)

```ebnf
program    = ( fn_decl | struct_decl | impl_decl | class_decl
             | enum_decl  | hash_const  | hash_import
             | trait_decl | impl_for_decl | extern_fn )* ;

hash_import    = '#import' STRING ';' ;                    // также: 'import'
hash_const     = '#const' IDENT ':' type '=' expr ';' ;   // также: 'const'
enum_decl      = 'enum'  IDENT '{' (IDENT ',')* '}' ;
trait_decl     = 'trait' IDENT '{' trait_method* '}' ;
trait_method   = ['pub'] ['static'] 'fn' IDENT '(' param_list ')' [(':' | '->') type] ';' ;
impl_for_decl  = 'impl' IDENT 'for' IDENT '{' method_decl* '}' ;
fn_decl        = ['@' IDENT] ['pub'] ['static'] 'fn' IDENT '(' param_list ')'
                 [(':' | '->') type] (block | '=>' expr ';') ;
extern_fn      = 'extern' 'fn' IDENT '(' param_list ')' ':' type ';' ;
class_decl     = ['class' IDENT ['extends' IDENT] '{' field_list method_list '}'] ;

block = '{' stmt* '}' ;
stmt  = let_stmt | mut_stmt | tuple_assign | assign | compound_assign
      | if_stmt | unless_stmt
      | while_stmt | do_while | for_stmt | repeat_stmt | loop_stmt
      | return_stmt | println_stmt | defer_stmt | match_stmt
      | field_assign | index_assign | assert_stmt | expr ';' ;

let_stmt       = 'let' IDENT [':' type] '=' expr ';' ;    // также: 'var'
mut_stmt       = 'mut' IDENT [':' type] '=' expr ';' ;

assert_stmt    = 'assert!' '(' expr ')' ';'
               | 'assert_eq!' '(' expr ',' expr ')' ';' ;

tuple_assign   = ('let' | 'var') '(' IDENT (',' IDENT)* ')' '=' '(' expr_list ')' ';' ;

for_stmt = 'for' IDENT 'in' (range_expr | IDENT) (',' IDENT 'in' range_expr)* block ;

expr      = pipe_expr ;
pipe_expr = elvis ('|>' (IDENT | lambda) ['(' arg_list ')'])* ;
elvis     = ternary ('?:' ternary)* ;
ternary   = or_expr ['?' or_expr ':' ternary] ;
or_expr   = and_expr ('||' and_expr)* ;
and_expr  = cmp_expr ('&&' cmp_expr)* ;
cmp_expr  = add_expr [('=='|'!='|'<'|'<='|'>'|'>=') add_expr] ;
add_expr  = mul_expr (('+' | '-') mul_expr)* ;
mul_expr  = unary (('*' | '/' | '%') unary)* ;
unary     = ('-' | '!' | '~') unary | power ;
power     = postfix ['**' unary] ;
postfix   = call_base ('.' IDENT ['(' arg_list ')'] | '[' expr ']'
                       | '?.' IDENT ['(' arg_list ')']
                       | '::' IDENT ['(' arg_list ')'])* ;
call_base = 'new' IDENT '(' arg_list ')'
          | IDENT '{' field_inits '}'
          | IDENT '!' '(' arg_list ')'           // макро-вызов println!, assert!
          | IDENT '(' arg_list ')'
          | '[' arg_list ']'
          | '(' arg_list ')'                     // кортеж
          | '|' param_list '|' (expr | block)   // лямбда
          | '&' IDENT | '*' IDENT
          | 'match' expr '{' match_arms '}'     // match как выражение
          | 'readInt' '(' ')' | 'readFloat' '(' ')'
          | primary ;
primary   = INT | FLOAT | STRING | INTERP_STRING | IDENT | 'self'
          | 'true' | 'false' | '(' expr ')' ;

compound_assign = IDENT ('+=' | '-=' | '*=' | '/=' | '%=' | '^=') expr ';' ;
```
