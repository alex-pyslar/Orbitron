# Orbitron

Компилируемый язык программирования с синтаксисом, вдохновлённым **Go, Rust, Java, C# и Kotlin**.
Компилируется в нативный бинарный файл через LLVM IR.

---

## Быстрый старт

### Требования

- Rust + Cargo
- LLVM 18 (`llc`)
- Clang
- WSL (на Windows)

### Сборка компилятора

```bash
cargo build --release
```

### Компиляция и запуск программы

```bash
# имя бинарника выводится из имени файла (fibonacci.ot → ./fibonacci)
./target/release/orbitron examples/fibonacci.ot
./fibonacci

# или явно задать имя через -o
./target/release/orbitron -o fib examples/fibonacci.ot
./fib
```

### Флаги компилятора

```
orbitron [опции] <файл.ot>

  -h, --help         Вывести справку и выйти
      --version      Вывести версию и выйти
  -o <файл>          Имя выходного бинарника (по умолчанию: имя файла без .ot)
      --emit-llvm    Сохранить LLVM IR в <output>.ll и не компилировать дальше
      --save-temps   Сохранить промежуточные файлы (.ll, .s)
  -v, --verbose      Выводить шаги компиляции
```

Пример:

```bash
cargo build --release
./target/release/orbitron examples/fibonacci.ot     # → ./fibonacci
./target/release/orbitron -v -o fib examples/fibonacci.ot && ./fib
./target/release/orbitron --emit-llvm examples/hello.ot  # → hello.ll
```

---

## Обзор языка

### Переменные и функции

```orbitron
var x = 42;
var pi: float = 3.14;

func add(a: int, b: int): int {
    return a + b;
}

func main() {
    println(add(x, 8)); // 50
}
```

### Ввод и вывод

```orbitron
println("Введите число:");
var n = readInt();
println(n * n);

var f = readFloat();
println(f * 2.0);
```

### Все виды циклов

```orbitron
// Диапазон исключительный (0, 1, 2, 3)
for i in 0..4 { println(i); }

// Диапазон включительный (0, 1, 2, 3, 4)
for i in 0..=4 { println(i); }

// Вложенный цикл одной строкой
for i in 1..=3, j in 1..=3 { println(i * j); }

// Условный цикл
while (x > 0) { x -= 1; }

// Цикл с постусловием
do { x += 1; } while (x < 5);

// Бесконечный цикл с break
loop { if (готово) { break; } }
```

### Сопоставление с образцом

```orbitron
match score {
    1 => { println("Бронза"); }
    2 => { println("Серебро"); }
    3 => { println("Золото"); }
    _ => { println("Нет медали"); }
}
```

### ООП — два стиля

**Стиль Go/Rust: `struct + impl`**

```orbitron
struct Vec2 {
    x: int,
    y: int,
}

impl Vec2 {
    pub func len_sq(self): int {
        return self.x * self.x + self.y * self.y;
    }
}

var v = Vec2 { x: 3, y: 4 }; // литерал без new
println(v.len_sq());           // 25
```

**Стиль Java/C#/Kotlin: `class + init`**

```orbitron
class Counter {
    private val:  int,
    private step: int,

    init(v: int, s: int) {
        self.val  = v;
        self.step = s;
    }

    pub func tick(self) { self.val = self.val + self.step; }
    pub func get(self): int { return self.val; }
}

var c = new Counter(0, 5);
c.tick();
println(c.get()); // 5
```

---

## Примеры

| Файл                          | Содержание                                     |
|-------------------------------|------------------------------------------------|
| `example.ot`                  | Полная демонстрация всех возможностей          |
| `examples/hello.ot`           | Привет, мир — базовые переменные               |
| `examples/fibonacci.ot`       | Числа Фибоначчи: рекурсия и итерация           |
| `examples/oop_struct.ot`      | Геометрия с `struct + impl`                    |
| `examples/oop_class.ot`       | Инкапсуляция с `class + init`                  |
| `examples/input_demo.ot`      | Ввод с клавиатуры: `readInt()`, `readFloat()`  |

---

## Документация

| Файл                      | Содержание                                 |
|---------------------------|--------------------------------------------|
| `docs/syntax.md`          | Полный справочник синтаксиса               |
| `docs/examples.md`        | Аннотированные примеры с объяснениями      |
| `docs/architecture.md`    | Устройство компилятора: лексер, парсер, кодогенерация |

---

## Структура проекта

```
Orbitron/
├── Cargo.toml
├── README.md
├── example.ot             ← демонстрация всех возможностей
├── examples/
│   ├── hello.ot
│   ├── fibonacci.ot
│   ├── oop_struct.ot
│   ├── oop_class.ot
│   └── input_demo.ot
├── docs/
│   ├── syntax.md
│   ├── examples.md
│   └── architecture.md
└── src/
    ├── main.rs
    ├── error.rs
    ├── lexer/
    │   ├── mod.rs
    │   └── token.rs
    ├── parser/
    │   ├── mod.rs
    │   └── ast.rs
    └── codegen/
        ├── mod.rs
        ├── expr.rs
        └── stmt.rs
```

---

## Пайплайн компиляции

```
.ot файл → Лексер → Парсер → AST → CodeGen → LLVM IR → llc → .s → clang → бинарник
```

---

## Лицензия

MIT
