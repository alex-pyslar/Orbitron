# Справочник синтаксиса — Orbitron

Orbitron — компилируемый язык с синтаксисом, вдохновлённым Go, Rust, Python, Ruby, Elixir,
Java, C# и Kotlin. Компилируется через LLVM в нативный бинарный файл.

---

## Содержание

1. [Переменные и типы](#1-переменные-и-типы)
2. [Константы](#2-константы)
3. [Функции](#3-функции)
4. [Вывод и ввод](#4-вывод-и-ввод)
5. [Строковая интерполяция](#5-строковая-интерполяция)
6. [Операторы](#6-операторы)
7. [Условия](#7-условия)
8. [Циклы](#8-циклы)
9. [Массивы](#9-массивы)
10. [Перечисления (enum)](#10-перечисления-enum)
11. [Сопоставление с образцом (match)](#11-сопоставление-с-образцом-match)
12. [Отложенный вызов (defer)](#12-отложенный-вызов-defer)
13. [Структуры (struct + impl)](#13-структуры-struct--impl)
14. [Классы (class)](#14-классы-class)
15. [Система проектов и импорт](#15-система-проектов-и-импорт)
16. [Приоритет операторов](#16-приоритет-операторов)
17. [Грамматика (EBNF)](#17-грамматика-ebnf)

---

## 1. Переменные и типы

```orbitron
var x = 42;           // целое число (int, 64-бит)
var pi: float = 3.14; // вещественное (float, 64-бит); аннотация типа — опциональна
var s = 10;
s = s + 1;            // переприсваивание без var
```

Поддерживаемые типы: `int` (i64), `float` (f64).

При смешанных арифметических операциях `int` автоматически повышается до `float`.

---

## 2. Константы  *(Rust / C++)*

```orbitron
const MAX: int   = 100;
const PI:  int   = 3;
const TAX: float = 0.2;

func main() {
    println(MAX);          // 100
    println(PI * 5 * 5);   // 75
}
```

- Объявляются на верхнем уровне или внутри функции.
- Значение — числовой литерал.
- Доступны во всех функциях того же файла (и в импортирующих файлах).

---

## 3. Функции

```orbitron
func add(a: int, b: int): int {
    return a + b;
}

func greet() {
    println("Hello!");
}
```

Аннотации типов у параметров и возвращаемого значения **опциональны** (служат документацией).

Точка входа — функция `main`:

```orbitron
func main() {
    println(add(2, 3)); // 5
}
```

---

## 4. Вывод и ввод

| Конструкция     | Описание                        |
|-----------------|---------------------------------|
| `println(выр);` | Вывод значения + перевод строки |
| `readInt()`     | Чтение целого числа из stdin    |
| `readFloat()`   | Чтение вещественного из stdin   |

```orbitron
println("Введите число:");
var n = readInt();
println(n * n);

var f = readFloat();
println(f * 2.0);
```

---

## 5. Строковая интерполяция  *(C# / Kotlin)*

Синтаксис `$"..."` позволяет встраивать переменные и константы прямо в строку:

```orbitron
var x     = 42;
var score = 100;
println($"x = {x}");           // x = 42
println($"score: {score}");    // score: 100
println($"PI = {PI}");         // PI = 3  (константа)
```

> Поддерживаются: переменные и константы типа `int` и `float`.
> Строковая интерполяция разрешена только внутри `println()`.

---

## 6. Операторы

### Арифметика

| Оператор | Значение                           |
|----------|------------------------------------|
| `+`      | Сложение                           |
| `-`      | Вычитание                          |
| `*`      | Умножение                          |
| `/`      | Деление                            |
| `%`      | Остаток от деления                 |
| `**`     | Возведение в степень *(Python)*    |

```orbitron
var p = 2 ** 10;    // 1024
var q = 3 ** 4;     // 81
```

### Сравнение

`==`  `!=`  `<`  `<=`  `>`  `>=`

Результат: `-1` (истина) или `0` (ложь) — оба представляются как `int`.

### Логика

`&&`  `||`  `!`

### Тернарный оператор  *(C / Java)*

```orbitron
var max = a > b ? a : b;
var abs = x >= 0 ? x : -x;

// Цепочка (право-ассоциативный):
var label = n > 10 ? 3 : n > 0 ? 2 : 1;
```

### Оператор канала `|>`  *(Elixir / F#)*

Передаёт значение левой части как первый аргумент функции правой части:

```orbitron
func double(n: int): int { return n * 2; }
func inc(n: int):    int { return n + 1; }

var result = 3 |> double |> inc;   // inc(double(3)) = 7
```

### Составное присваивание

| Форма      | Эквивалент    |
|------------|---------------|
| `x += 5;`  | `x = x + 5;`  |
| `x -= 3;`  | `x = x - 3;`  |
| `x *= 2;`  | `x = x * 2;`  |
| `x /= 4;`  | `x = x / 4;`  |

---

## 7. Условия

```orbitron
if (условие) {
    // ...
} else if (другое) {
    // ...
} else {
    // ...
}
```

### `unless`  *(Ruby)*

Выполняется, когда условие **ложно** — синтаксический сахар для `if (!...)`:

```orbitron
unless (x == 0) {
    println(100 / x);   // безопасное деление
}
```

---

## 8. Циклы

### `for..in` — диапазонный цикл

```orbitron
// Исключительный диапазон [from, to)
for i in 0..4 {
    println(i);   // 0 1 2 3
}

// Включительный диапазон [from, to]
for i in 0..=4 {
    println(i);   // 0 1 2 3 4
}
```

### Многодиапазонный `for` — вложенные циклы одной строкой

```orbitron
// Эквивалентно: for i { for j { ... } }
for i in 0..3, j in 0..3 {
    println(i * 10 + j);
}
```

### `while` — цикл с предусловием

```orbitron
while (n > 0) {
    n -= 1;
}
```

### `do..while` — цикл с постусловием

```orbitron
do {
    n += 1;
} while (n < 10);
```

### `loop` — бесконечный цикл

```orbitron
loop {
    if (done) { break; }
}
```

### `repeat N`  *(Lua / Pascal)*

Повторить тело ровно N раз:

```orbitron
repeat 5 {
    println("Hello!");
}

var counter = 0;
repeat 10 { counter += 1; }
// counter == 10
```

### `break` и `continue`

```orbitron
for i in 0..10 {
    if (i == 5)      { break; }
    if (i % 2 == 0)  { continue; }
    println(i);    // 1 3
}
```

---

## 9. Массивы  *(Python / JavaScript)*

```orbitron
var primes = [2, 3, 5, 7, 11];

// Чтение
println(primes[0]);    // 2
println(primes[4]);    // 11

// Запись
primes[0] = 99;

// Обход
for i in 0..5 {
    println(primes[i]);
}

// Накопление
var sum = 0;
for i in 0..5 { sum += primes[i]; }
```

Элементы массива — `int`. Размер определяется при инициализации.

---

## 10. Перечисления (`enum`)  *(Rust / Swift)*

```orbitron
enum Color  { Red, Green, Blue }
enum Season { Spring, Summer, Autumn, Winter }
```

Каждый вариант получает целочисленное значение: 0, 1, 2, ...

```orbitron
var c = Color.Red;      // c == 0
var s = Season.Summer;  // s == 1
println(s);             // 1
```

---

## 11. Сопоставление с образцом (`match`)

```orbitron
match выражение {
    значение       => { /* блок */ }
    EnumName.Var   => { /* вариант enum */ }
    _              => { /* по умолчанию */ }
}
```

Образцы: целые числа, варианты `enum`, `_` (wildcard).

```orbitron
enum Dir { North, South, East, West }
var d = Dir.East;

match d {
    Dir.North => { println("Север"); }
    Dir.East  => { println("Восток"); }
    _         => { println("Другое"); }
}
```

---

## 12. Отложенный вызов (`defer`)  *(Go)*

`defer` регистрирует оператор для выполнения **прямо перед выходом из функции**.
При нескольких `defer` выполняются в порядке LIFO (последний — первым).

```orbitron
func example() {
    defer println("Конец!");   // напечатается последним
    println("Начало");
    println("Середина");
}
// Вывод: Начало → Середина → Конец!
```

---

## 13. Структуры (`struct + impl`)

Стиль Go / Rust. Данные и методы определяются отдельно.

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

// Создание: литерал без `new`
var p = Point { x: 3, y: 4 };
println(p.dist_sq());   // 25
p.move_by(1, 0);
```

Поля: `name: int` или `name: float`.
`self` — явный первый параметр всех методов.

---

## 14. Классы (`class`)

Стиль Java / C# / Kotlin. Данные и методы объединены, есть конструктор `init`.

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

### Модификаторы доступа

| Ключевое слово | Значение                 |
|----------------|--------------------------|
| `pub`          | Публичный (по умолчанию) |
| `private`      | Приватный                |

### Сравнение стилей ООП

| Аспект       | `struct + impl`            | `class`                   |
|--------------|----------------------------|---------------------------|
| Вдохновение  | Go, Rust                   | Java, C#, Kotlin          |
| Создание     | `Foo { field: val }`       | `new Foo(args)`           |
| Конструктор  | не нужен                   | `init(params) { ... }`    |
| Методы       | в блоке `impl Foo { ... }` | внутри `class Foo { ... }`|
| `self`       | явный параметр             | явный параметр            |

---

## 15. Система проектов и импорт

### Структура проекта

```
myproject/
├── orbitron.toml       # манифест проекта
├── src/
│   ├── main.ot         # точка входа (содержит func main)
│   ├── math.ot         # модуль
│   └── geometry.ot     # ещё один модуль
└── bin/                # директория выходных бинарников
```

### Манифест `orbitron.toml`

```toml
[project]
name    = "myproject"
version = "0.1.0"

[build]
main   = "src/main.ot"     # точка входа
output = "bin/myproject"   # путь к выходному бинарнику
```

### Импорт модулей

```orbitron
// src/main.ot
import "math";       // загружает src/math.ot
import "geometry";   // загружает src/geometry.ot

func main() {
    println(add(2, 3));   // функция из math.ot
}
```

Импорт разрешается **до** кодогенерации: компилятор обходит все файлы и объединяет AST.
Циклические импорты → ошибка компиляции. Один файл не импортируется дважды.

### CLI-команды

```bash
# Создать новый проект
orbitron new myapp
cd myapp

# Собрать (ищет orbitron.toml вверх по дереву директорий)
orbitron build

# Собрать и запустить
orbitron run

# Флаги
orbitron build -o bin/release      # имя выходного файла
orbitron build --emit-llvm         # сохранить .ll и остановиться
orbitron build --save-temps        # сохранить .ll и .s
orbitron build -v                  # подробный вывод шагов
```

### Однофайловый режим (обратная совместимость)

```bash
orbitron hello.ot                  # → ./hello
orbitron -o myapp hello.ot         # → ./myapp
orbitron --emit-llvm hello.ot      # → hello.ll (без линковки)
orbitron -v examples/fib.ot        # подробный вывод
```

---

## 16. Приоритет операторов

От низкого к высокому:

| Уровень | Операторы                         | Ассоциативность |
|---------|-----------------------------------|-----------------|
| 1       | `\|>`                             | левая           |
| 2       | `? :`                             | правая          |
| 3       | `\|\|`                            | левая           |
| 4       | `&&`                              | левая           |
| 5       | `== != < <= > >=`                 | —               |
| 6       | `+ -`                             | левая           |
| 7       | `* / %`                           | левая           |
| 8       | `- !` (унарные)                   | правая          |
| 9       | `**`                              | правая          |
| 10      | `.field` `.method()` `[idx]`      | левая (постфикс)|

---

## 17. Грамматика (EBNF, упрощённо)

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

## Специальные значения

| Литерал | Значение  |
|---------|-----------|
| `true`  | `1` (int) |
| `false` | `0` (int) |

---

## Строки

Строковые литералы (`"..."`) допустимы **только** внутри `println()`.
Для встройки переменных используйте `$"..."`.

```orbitron
println("Любой текст");
println("Строка с \"кавычками\"");
println($"x = {x}");
```

---

## Комментарии

```orbitron
// Однострочный комментарий

/* Многострочный
   комментарий */
```
