# Orbitron

**Компилируемый язык программирования с двумя бэкендами и выразительным современным синтаксисом.**

Orbitron компилирует файлы `.ot` в нативные бинарники через LLVM IR или в кросс-платформенный байткод JVM (`.jar`). Синтаксис сочетает лучшие идеи из Rust, Go, Kotlin, Python, Elixir и Java в единой системе.

```orbitron
fn main() {
    var name = "Orbitron";
    var mut version = 2;
    println($"Добро пожаловать в {name} v{version}!");

    var primes = [2, 3, 5, 7, 11];
    var mut sum = 0;
    for i in 0..5 { sum += primes[i]; }
    println($"Сумма первых 5 простых: {sum}");
}
```

---

## Возможности языка

| Возможность | Синтаксис | Источник |
|---|---|---|
| Неизменяемая переменная | `var x = 42;` | Rust |
| Изменяемая переменная | `var mut x = 42;` | Rust |
| Константа | `const MAX: i64 = 100;` | Rust, C++ |
| Интерполяция строк | `$"val={x}"` | C#, Kotlin |
| Степень | `2 ** 10` | Python |
| Конвейер | `x \|> double \|> inc` | Elixir, F# |
| `unless` | `unless (x == 0) { }` | Ruby |
| Массивы | `var a = [1, 2, 3];` | Python, JS |
| Перечисления | `enum Dir { North, South }` | Rust, Swift |
| Отложенное выполнение | `defer println("done");` | Go |
| Повтор N раз | `repeat 5 { }` | Lua, Pascal |
| Тернарный оператор | `a > b ? a : b` | C, Java |
| Структуры + методы | `struct Foo { }  impl Foo { }` | Go, Rust |
| Классы | `class Foo { init() { } }` | Java, Kotlin |
| Модификаторы доступа | `public / private / protected / internal` | Java, Kotlin |
| Статические методы | `static fn method()` · `Type::method()` | Java, Kotlin |
| Трейты | `trait Foo { }  impl Foo for Bar { }` | Rust, Swift |
| Лямбды | `\|x\| x * 2` | Rust |
| Кортежи | `var (a, b) = (1, 2);` | Python, Rust |
| Горутины | `go { ... }` | Go |
| Async/await | `async fn f()` · `await expr` | Kotlin, Rust |
| Каналы | `var ch = chan(); ch <- 42; var v = <-ch;` | Go |
| Системные вызовы | `syscall(SYS_WRITE, STDOUT, buf, n);` | C |
| Внешние C-функции | `extern fn socket(...): i64;` | C |

---

## Быстрый старт

```bash
# Сборка компилятора (требует Rust + LLVM 18)
cargo build --release

# Один файл — LLVM бэкенд
./target/release/orbitron examples/01_basics/hello.ot && ./hello

# Один файл — JVM бэкенд
./target/release/orbitron examples/01_basics/hello.ot --backend jvm && java -jar hello.jar

# Новый проект
./target/release/orbitron new myapp
cd myapp && ../target/release/orbitron run
```

**На Windows — через WSL:**

```bash
wsl -e bash -c "cd /mnt/c/source/Orbitron && cargo build --release 2>&1"
wsl -e bash -c "cd /mnt/c/source/Orbitron && ./target/release/orbitron examples/01_basics/hello.ot && ./hello"
```

---

## Установка

### Требования

| Инструмент | Назначение |
|---|---|
| Rust + Cargo | Сборка компилятора |
| LLVM 18 (`llc`, `clang`) | Бэкенд LLVM — нативный бинарник |
| JDK 11+ (`javac`, `jar`) | Бэкенд JVM — выходной `.jar` |
| `libm` | Оператор `**` (степень) |

```bash
git clone https://github.com/alex-pyslar/Orbitron
cd Orbitron
cargo build --release
export ORBITRON_HOME=/path/to/Orbitron   # путь к stdlib/
```

---

## Краткий тур по языку

### Переменные

```orbitron
var x = 42;              // неизменяемая (i64)
var mut count = 0;       // изменяемая
var pi: f64 = 3.14;      // с аннотацией типа

const MAX: i64 = 1000;   // константа времени компиляции
const TAX: f64 = 0.2;
```

### Функции

```orbitron
fn add(a: i64, b: i64) -> i64 {
    return a + b;
}

fn double(n: i64) => n * 2;   // однострочная форма

fn main() {
    println(add(10, 20));  // 30
    println(3 |> double);  // 6
}
```

### Управление потоком

```orbitron
if (score >= 90) {
    println("Отлично");
} else if (score >= 70) {
    println("Хорошо");
} else {
    println("Продолжай");
}

unless (x == 0) { println(100 / x); }   // выполняется если условие ЛОЖНО

match s {
    Status.Ok      => { println("ок");      }
    Status.Error   => { println("ошибка");  }
    _              => { println("другое");  }
}
```

### Циклы

```orbitron
for i in 0..10 { }              // 0–9
for i in 0..=10 { }             // 0–10 включительно
for x in arr { println(x); }    // итерация по массиву

while (n > 0) { n -= 1; }
do { n += 1; } while (n < 10);
repeat 5 { counter += 1; }
```

### Структуры — стиль Go/Rust

```orbitron
struct Point {
    public var x: i64,
    public var y: i64,
    static var count: i64,
}

impl Point {
    public static fn new(x: i64, y: i64) -> Point {
        Point::count += 1;
        return Point { x, y };
    }
    public fn dist_sq(self) -> i64 => self.x * self.x + self.y * self.y;
    private fn reset(self) { self.x = 0; self.y = 0; }
}

var p = Point::new(3, 4);
println(p.dist_sq());   // 25
```

### Классы — стиль Java/Kotlin

```orbitron
class BankAccount {
    private var balance: i64,

    init(initial: i64) { self.balance = initial; }

    public fn deposit(self, amount: i64) {
        if (amount > 0) { self.balance += amount; }
    }

    public fn withdraw(self, amount: i64) -> i64 {
        if (self.balance >= amount) {
            self.balance -= amount;
            return 1;
        }
        return 0;
    }

    public fn get_balance(self) -> i64 => self.balance;
}

var acc = new BankAccount(500);
acc.deposit(200);
println(acc.get_balance());   // 700
```

### Трейты и оператор-перегрузки

```orbitron
trait Drawable {
    fn draw(self);
    fn area(self) -> i64;
}

impl Drawable for Circle { ... }
impl Drawable for Square { ... }

impl Add for Vec2 {
    fn add(self, other: Vec2) -> Vec2 {
        return Vec2 { x: self.x + other.x, y: self.y + other.y };
    }
}
```

### Горутины и каналы

```orbitron
// Горутина — запуск в фоне
go { println("параллельно"); };

// Каналы — передача значений между горутинами
var ch = chan();
go { ch <- 42; };
var val = <-ch;
println($"получено: {val}");
```

### Async / Await

```orbitron
async fn compute(n: i64) -> i64 {
    var mut result = 0;
    for i in 0..n { result += i; }
    return result;
}

fn main() {
    var result = await compute(100);
    println(result);   // 4950
}
```

### Defer

```orbitron
fn process() {
    defer println("очистка");   // выполняется последним (LIFO)
    defer println("закрытие");
    println("работа...");
    // вывод: работа... → закрытие → очистка
}
```

### Стандартная библиотека

```orbitron
import "std/math";
import "std/bits";
import "std/algo";

fn main() {
    println(factorial(10));       // 3628800
    println(gcd(48, 18));         // 6
    println(is_prime(97));        // 1
    println(bit_count(255));      // 8
    println(next_pow2(5));        // 8
    println(ipow(2, 10));         // 1024
    println(isqrt(144));          // 12
}
```

---

## CLI

```
ИСПОЛЬЗОВАНИЕ:
  orbitron new <имя>             Создать новый проект
  orbitron build [опции]         Собрать проект (читает orbitron.toml)
  orbitron run   [опции]         Собрать и запустить
  orbitron fmt   [файлы]         Форматировать исходники
  orbitron <file.ot> [опции]     Скомпилировать один файл

ОПЦИИ:
  -o <файл>               Имя выходного файла
      --backend llvm|jvm  Бэкенд (по умолчанию: llvm)
      --emit-llvm         Сохранить LLVM IR и остановиться
      --emit-java         Сохранить Java-исходник и остановиться
      --save-temps        Сохранить промежуточные файлы
  -v, --verbose           Подробный вывод
  -h, --help              Справка
```

---

## Стандартная библиотека

| Модуль | Импорт | Содержимое |
|---|---|---|
| math | `import "std/math"` | `abs`, `max`, `min`, `factorial`, `fib`, `gcd`, `is_prime`, `PI`, `E` |
| bits | `import "std/bits"` | `bit_count`, `bit_len`, `is_pow2`, `next_pow2`, `shl`, `shr` |
| algo | `import "std/algo"` | `min3`, `max3`, `lerp`, `map_range`, `ipow`, `isqrt`, `digit_sum` |
| sys  | `import "std/sys"`  | Константы Linux syscall, `sys_alloc`, `sys_free`, `sys_write` |
| net  | `import "std/net"`  | `tcp_connect`, `net_send`, `net_recv`, `tcp_listen`, `tcp_accept` |
| db   | `import "std/db"`   | Объявления SQLite3 (требует `-lsqlite3`) |

---

## Конвейер компиляции

```
┌─────────────────────────────────────────────────────┐
│                   Бэкенд LLVM                        │
│                                                      │
│  source.ot → Lexer → Parser → Resolver               │
│                                    │                 │
│                           Слитый AST                 │
│                                    │                 │
│                              CodeGen → LLVM IR (.ll) │
│                                            │         │
│                                   llc → Ассемблер    │
│                                            │         │
│                                 clang -lm → Бинарник │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│                   Бэкенд JVM                         │
│                                                      │
│  source.ot → Lexer → Parser → Resolver → JvmCodeGen  │
│                                               │      │
│                                  Main.java ←──┘      │
│                                       │              │
│                              javac + jar → .jar      │
└─────────────────────────────────────────────────────┘
```

---

## Структура репозитория

```
src/
├── main.rs          Диспетчер CLI
├── cli.rs           Разбор аргументов, enum Backend
├── pipeline.rs      Конвейеры компиляции
├── resolver.rs      Рекурсивный резолвер импортов
├── lexer/
│   ├── mod.rs       Лексер
│   └── token.rs     Токены и ключевые слова
├── parser/
│   ├── mod.rs       Рекурсивный нисходящий парсер
│   └── ast.rs       Узлы AST (Expr, Stmt, ...)
├── codegen/
│   ├── mod.rs       CodeGen, трёхпроходная кодогенерация
│   ├── expr.rs      Выражения → LLVM IR
│   └── stmt.rs      Инструкции → LLVM IR
├── jvm/
│   └── mod.rs       AST → Java → javac → jar
└── fmt/
    └── mod.rs       AST pretty-printer (orbitron fmt)

stdlib/              Стандартная библиотека (на самом Orbitron)
docs/                Документация — книга на русском (12 глав)
examples/            Примеры, организованные по темам (01–08)
```

---

## Примеры

| Папка | Тема |
|---|---|
| `examples/01_basics/` | Переменные, операторы, ввод, Hello World |
| `examples/02_control_flow/` | if / match / все циклы |
| `examples/03_functions/` | Функции, рекурсия, конвейер `\|>` |
| `examples/04_collections/` | Массивы, enum, сортировка, кортежи |
| `examples/05_oop/` | struct+impl, class, трейты, наследование |
| `examples/06_stdlib/` | Демо math / bits / algo |
| `examples/07_advanced/` | Новые фичи, syscall, сеть |
| `examples/08_projects/` | Многофайловые проекты (calculator, geometry) |
| `examples/concurrency.ot` | Горутины, каналы, async/await |

---

## Лицензия

MIT — смотрите [LICENSE](LICENSE).
