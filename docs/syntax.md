# Справочник синтаксиса языка Orbitron

Orbitron — компилируемый язык с синтаксисом, вдохновлённым Go, Rust, Java, C# и Kotlin.
Компилируется через LLVM в нативный бинарный файл.

---

## Переменные

```orbitron
var x = 42;           // целое число (int)
var pi: float = 3.14; // вещественное (float), аннотация типа опциональна
var s = 10;
s = s + 1;            // переприсваивание без var
```

Поддерживаемые типы: `int` (64-битное целое), `float` (64-битное вещественное).

---

## Функции

```orbitron
func имя(параметр: тип, ...): тип_возврата {
    return выражение;
}
```

Аннотации типов у параметров и возвращаемого значения опциональны и не проверяются при компиляции — служат документацией.

```orbitron
func add(a: int, b: int): int {
    return a + b;
}

func greet() {
    println("Привет!");
}
```

Точка входа — функция `main`:

```orbitron
func main() {
    println(add(2, 3)); // 5
}
```

---

## Вывод и ввод

| Конструкция        | Описание                        |
|--------------------|---------------------------------|
| `println(выр);`    | Вывод значения и перевод строки |
| `readInt()`        | Чтение целого числа из stdin    |
| `readFloat()`      | Чтение дробного числа из stdin  |

```orbitron
println("Введите число:");
var n = readInt();
println(n * n);

var f = readFloat();
println(f * 2.0);
```

---

## Операторы

### Арифметические
| Оператор | Значение      |
|----------|---------------|
| `+`      | Сложение      |
| `-`      | Вычитание     |
| `*`      | Умножение     |
| `/`      | Деление       |
| `%`      | Остаток       |

### Сравнение
`==`  `!=`  `<`  `<=`  `>`  `>=`

Результат: `-1` (истина) или `0` (ложь).

### Логические
`&&`  `||`  `!`

### Присваивание
| Форма       | Эквивалент      |
|-------------|-----------------|
| `x += 5;`   | `x = x + 5;`   |
| `x -= 3;`   | `x = x - 3;`   |
| `x *= 2;`   | `x = x * 2;`   |
| `x /= 4;`   | `x = x / 4;`   |

---

## Условный оператор

```orbitron
if (условие) {
    // ...
} else if (другое_условие) {
    // ...
} else {
    // ...
}
```

---

## Циклы

### `for..in` — диапазонный цикл

```orbitron
// Исключительный диапазон: i = 0, 1, 2, 3
for i in 0..4 {
    println(i);
}

// Включительный диапазон: i = 0, 1, 2, 3, 4
for i in 0..=4 {
    println(i);
}
```

### Вложенный цикл одной строкой

```orbitron
// Эквивалентно двум вложенным for
for i in 0..3, j in 0..3 {
    println(i * 10 + j);
}
```

### `while` — цикл с предусловием

```orbitron
while (условие) {
    // ...
}
```

### `do..while` — цикл с постусловием

```orbitron
do {
    // ...
} while (условие);
```

### `loop` — бесконечный цикл

```orbitron
loop {
    // ...
    if (условие_выхода) { break; }
}
```

### `break` и `continue`

```orbitron
for i in 0..10 {
    if (i == 5) { break; }    // выйти из цикла
    if (i % 2 == 0) { continue; } // перейти к следующей итерации
    println(i);
}
```

---

## Сопоставление с образцом

```orbitron
match выражение {
    значение => { /* блок */ }
    значение => { /* блок */ }
    _        => { /* блок по умолчанию */ }
}
```

Образцы: целые числа и `_` (wildcard).

```orbitron
var код = 2;
match код {
    1 => { println("Бронза"); }
    2 => { println("Серебро"); }
    3 => { println("Золото"); }
    _ => { println("Нет медали"); }
}
```

---

## Структуры (`struct + impl`)

Стиль Go/Rust. Данные и методы определяются отдельно.

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
println(p.dist_sq()); // 25
p.move_by(1, 0);
```

Поля структуры: `имя: int` или `имя: float`.

---

## Классы (`class`)

Стиль Java/C#/Kotlin. Данные и методы объединены, есть конструктор `init`.

```orbitron
class Counter {
    private val:  int,
    private step: int,

    // Конструктор (self не пишется явно)
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

// Создание: new ClassName(аргументы)
var c = new Counter(0, 5);
c.tick();
println(c.get()); // 5
```

### Модификаторы доступа

| Ключевое слово | Значение                  |
|----------------|---------------------------|
| `pub`          | Публичный (по умолчанию)  |
| `private`      | Приватный                 |

Модификаторы доступа парсятся, но не проверяются при кодогенерации (будет добавлено в будущем).

---

## Два стиля ООП — сравнение

| Аспект          | `struct + impl`               | `class`                        |
|-----------------|-------------------------------|--------------------------------|
| Вдохновение     | Go, Rust                      | Java, C#, Kotlin               |
| Создание        | `Foo { field: val }`          | `new Foo(args)`                |
| Конструктор     | не нужен / отдельный `fn`     | `init(params) { ... }`         |
| Методы          | в блоке `impl Foo { ... }`    | внутри блока `class Foo { ... }` |
| `self`          | явный параметр первым         | явный параметр первым          |

---

## Специальные значения

| Литерал    | Значение      |
|------------|---------------|
| `true`     | `1` (int)     |
| `false`    | `0` (int)     |

---

## Строки

Строковые литералы (`"..."`) допустимы **только** внутри `println()`:

```orbitron
println("Любой текст");
println("Строка с \"кавычками\"");
```

---

## Комментарии

```orbitron
// Однострочный комментарий
```

Многострочные комментарии не поддерживаются.

---

## Грамматика (EBNF, упрощённо)

```
program    = (func_decl | struct_decl | impl_decl | class_decl)* ;

func_decl  = 'func' IDENT '(' param_list ')' [':' type] block ;
param_list = (IDENT [':' type] (',' IDENT [':' type])*)? ;

block      = '{' stmt* '}' ;
stmt       = var_stmt | assign | if_stmt | while_stmt | do_while
           | for_stmt | loop_stmt | return_stmt | println_stmt
           | match_stmt | field_assign | compound_assign | expr ';' ;

expr       = or_expr ;
or_expr    = and_expr ('||' and_expr)* ;
and_expr   = cmp_expr ('&&' cmp_expr)* ;
cmp_expr   = add_expr [('=='|'!='|'<'|'<='|'>'|'>=') add_expr] ;
add_expr   = mul_expr (('+' | '-') mul_expr)* ;
mul_expr   = unary (('*' | '/' | '%') unary)* ;
unary      = ('-' | '!') unary | postfix ;
postfix    = call_base ('.' IDENT ['(' arg_list ')'])* ;
call_base  = 'new' IDENT '(' arg_list ')'
           | IDENT '(' arg_list ')'
           | IDENT '{' field_inits '}'
           | 'readInt' '(' ')'
           | 'readFloat' '(' ')'
           | primary ;
primary    = INT | FLOAT | STRING | IDENT | 'self' | 'true' | 'false'
           | '(' expr ')' ;
```
