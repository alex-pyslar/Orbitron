# Orbitron

**Компилируемый язык программирования с двумя бэкендами и чистым выразительным синтаксисом.**

Orbitron компилирует файлы `.ot` в нативные бинарники через LLVM IR или в кросс-платформенный байткод JVM (`.jar`). Синтаксис сочетает лучшие идеи из Go, Rust, Python, Ruby, Elixir, C#, Kotlin и Java в единой системе.

```orbitron
func main() {
    var name = "Orbitron";
    var version = 2;
    println($"Добро пожаловать в {name} v{version}!");

    var primes = [2, 3, 5, 7, 11];
    var sum = 0;
    for i in 0..5 { sum += primes[i]; }
    println($"Сумма первых 5 простых: {sum}");
}
```

---

## Особенности языка

| Возможность | Синтаксис | Источник вдохновения |
|-------------|-----------|----------------------|
| Переменные | `var x = 42;` | |
| Константы | `const MAX: int = 100;` | Rust, C++ |
| Интерполяция строк | `$"val = {x}"` | C#, Kotlin |
| Оператор степени | `2 ** 10` | Python |
| Оператор конвейера | `x \|> double \|> inc` | Elixir, F# |
| `unless` — инверсный if | `unless (x == 0) { }` | Ruby |
| Массивы | `var a = [1, 2, 3];` | Python, JS |
| Перечисления (enum) | `enum Dir { North, South }` | Rust, Swift |
| Отложенное выполнение | `defer println("готово");` | Go |
| Повтор N раз | `repeat 5 { }` | Lua, Pascal |
| Тернарный оператор | `a > b ? a : b` | C, Java |
| Структуры + методы | `struct Foo { } impl Foo { }` | Go, Rust |
| Классы | `class Foo { init() { } }` | Java, Kotlin |
| Прямые системные вызовы | `syscall(SYS_WRITE, STDOUT, buf, n);` | C |
| Внешние C-функции | `extern func socket(...): int;` | C |

---

## Быстрый старт

```bash
# Сборка компилятора (требует Rust + LLVM 18)
cargo build --release

# Компиляция и запуск одного файла
./target/release/orbitron examples/01_basics/hello.ot && ./hello

# Создание нового проекта
./target/release/orbitron new myapp
cd myapp
../target/release/orbitron run
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
|-----------|-----------|
| Rust + Cargo | Сборка компилятора |
| LLVM 18 (`llc`, `clang`) | Бэкенд LLVM — нативный бинарник |
| JDK 11+ (`javac`, `jar`) | Бэкенд JVM — выходной `.jar` |
| `libm` | Математические операции (оператор `**`) |

### Сборка из исходников

```bash
git clone https://github.com/alex-pyslar/Orbitron
cd Orbitron
cargo build --release
```

Бинарник компилятора: `target/release/orbitron`.

Стандартная библиотека находится в `stdlib/` рядом с бинарником, или задайте переменную окружения:

```bash
export ORBITRON_HOME=/path/to/Orbitron
```

---

## Краткий тур по языку

### Переменные и типы

```orbitron
var x = 42;           // int (i64)
var pi: float = 3.14; // float (f64) — аннотация типа необязательна
var flag = true;      // true == 1, false == 0

const MAX: int   = 1000;
const TAX: float = 0.2;
```

### Функции

```orbitron
func add(a: int, b: int): int {
    return a + b;
}

func main() {
    println(add(10, 20));  // 30
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

// unless — выполняется когда условие ЛОЖНО (вдохновлён Ruby)
unless (x == 0) {
    println(100 / x);  // безопасное деление
}

// match — сопоставление по целым числам и enum
enum Status { Ok, Error, Pending }
var s = Status.Ok;

match s {
    Status.Ok      => { println("всё хорошо"); }
    Status.Error   => { println("ошибка!");    }
    Status.Pending => { println("ожидание");   }
    _              => { println("неизвестно"); }
}
```

### Циклы

```orbitron
for i in 0..10 { }             // исключающий диапазон: 0–9
for i in 0..=10 { }            // включающий диапазон: 0–10
for i in 0..3, j in 0..3 { }   // вложенные циклы в одну строку

while (n > 0) { n -= 1; }

do { n += 1; } while (n < 10);

loop { if (done) { break; } }

repeat 5 { counter += 1; }     // ровно 5 раз (из Lua / Pascal)
```

### Массивы

```orbitron
var primes = [2, 3, 5, 7, 11, 13];

println(primes[0]);    // 2
primes[0] = 99;        // изменение

var sum = 0;
for i in 0..6 { sum += primes[i]; }
println(sum);          // 138
```

### Интерполяция строк

```orbitron
var score = 95;
var player = 42;
println($"Счёт игрока {player}: {score}");
```

### Оператор конвейера `|>`

```orbitron
func double(n: int): int { return n * 2; }
func inc(n: int):    int { return n + 1; }
func square(n: int): int { return n * n; }

// Композиция функций слева направо
var result = 3 |> double |> inc |> square;  // ((3*2)+1)^2 = 49
println(result);
```

### Структуры — стиль Go/Rust

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

// Литерал структуры — без ключевого слова new!
var p = Point { x: 3, y: 4 };
println(p.dist_sq());   // 25
p.move_by(1, -1);
println(p.dist_sq());   // 13
```

### Классы — стиль Java/Kotlin

```orbitron
class BankAccount {
    private balance: int,

    init(initial: int) {
        self.balance = initial;
    }

    pub func deposit(self, amount: int) {
        if (amount > 0) { self.balance += amount; }
    }

    pub func withdraw(self, amount: int): int {
        if (self.balance >= amount) {
            self.balance -= amount;
            return 1;
        }
        return 0;  // недостаточно средств
    }

    pub func get_balance(self): int {
        return self.balance;
    }
}

var acc = new BankAccount(500);
acc.deposit(200);
println(acc.get_balance());   // 700
println(acc.withdraw(300));   // 1 (успех)
println(acc.get_balance());   // 400
```

### Defer

```orbitron
func process() {
    defer println("очистка");   // выполняется последним (порядок LIFO)
    defer println("закрытие");  // выполняется предпоследним

    println("работа...");
    // Порядок вывода: работа... → закрытие → очистка
}
```

### Стандартная библиотека

```orbitron
import "std/math";
import "std/bits";
import "std/algo";

func main() {
    println(factorial(10));          // 3628800
    println(gcd(48, 18));            // 6
    println(is_prime(97));           // 1
    println(sum_range(1, 100));      // 5050

    println(bit_count(255));         // 8
    println(is_pow2(16));            // 1
    println(next_pow2(5));           // 8

    println(ipow(2, 10));            // 1024
    println(isqrt(144));             // 12
    println(is_palindrome_num(121)); // 1
}
```

---

## Справочник CLI

```
ИСПОЛЬЗОВАНИЕ:
  orbitron new <имя>             Создать новый проект
  orbitron build [опции]         Собрать проект (читает orbitron.toml)
  orbitron run   [опции]         Собрать и запустить проект
  orbitron <file.ot> [опции]     Компилировать один файл

ОПЦИИ:
  -h, --help              Показать справку и выйти
      --version           Показать версию и выйти
  -o <файл>               Имя выходного файла
      --backend llvm|jvm  Бэкенд компиляции (по умолчанию: llvm)
      --emit-llvm         Сохранить LLVM IR (.ll) и остановиться
      --emit-java         Сохранить Java-исходник (.java) и остановиться
      --save-temps        Сохранить промежуточные файлы (.ll, .s)
  -v, --verbose           Подробный вывод шагов компиляции

БЭКЕНДЫ:
  llvm  -> нативный бинарник  (требует llc + clang + libm)
  jvm   -> файл .jar           (требует javac + jar; запуск: java -jar)
```

**Примеры:**

```bash
orbitron new myapp                   # создать проект
cd myapp && orbitron run             # собрать + запустить (LLVM)
orbitron hello.ot                    # один файл (LLVM)
orbitron hello.ot --backend jvm      # один файл (JVM)
orbitron build --emit-llvm           # посмотреть LLVM IR
orbitron build -v                    # подробный вывод
```

---

## Конфигурация проекта

```
myproject/
├── orbitron.toml
├── src/
│   ├── main.ot       # точка входа — должна содержать func main()
│   └── utils.ot      # модуль (import "utils";)
└── bin/              # скомпилированный вывод
```

**orbitron.toml:**

```toml
[project]
name    = "myapp"
version = "0.1.0"

[build]
main    = "src/main.ot"
output  = "bin/myapp"
backend = "llvm"          # или "jvm"
```

---

## Стандартная библиотека

| Модуль | Импорт | Содержимое |
|--------|--------|-----------|
| math | `import "std/math"` | `abs`, `max`, `min`, `clamp`, `factorial`, `fib`, `gcd`, `lcm`, `sign`, `is_prime`, `PI`, `E`, `INT_MAX` |
| bits | `import "std/bits"` | `bit_count`, `bit_len`, `is_pow2`, `next_pow2`, `prev_pow2`, `shl`, `shr`, `floor_log2`, `reverse_bits` |
| algo | `import "std/algo"` | `min3`, `max3`, `median3`, `lerp`, `map_range`, `dist`, `ipow`, `isqrt`, `digit_sum`, `is_palindrome_num`, `cycle` |
| sys  | `import "std/sys"` | Константы системных вызовов Linux, `sys_alloc`, `sys_free`, `sys_write`, `sys_sleep`, `sys_getpid` |
| net  | `import "std/net"` | `net_socket_tcp/udp`, `tcp_connect`, `net_send`, `net_recv`, `tcp_bind`, `tcp_listen`, `tcp_accept` |
| db   | `import "std/db"` | Объявления SQLite3 (требует флага `-lsqlite3`) |

---

## Низкоуровневое программирование

Orbitron предоставляет арифметику указателей, прямые системные вызовы Linux и объявления внешних C-функций:

```orbitron
import "std/sys";

func main() {
    // Выделение памяти
    var buf = sys_alloc(64);

    // Побайтовая запись по адресу
    ptr_write_byte(buf,     79);   // 'O'
    ptr_write_byte(buf + 1, 114);  // 'r'
    ptr_write_byte(buf + 2, 98);   // 'b'
    ptr_write_byte(buf + 3, 10);   // '\n'

    // Прямой системный вызов Linux: write(stdout, buf, 4)
    syscall(SYS_WRITE, STDOUT, buf, 4);

    sys_free(buf, 64);
}
```

```orbitron
// Объявление внешних C-функций
extern func open(path: int, flags: int): int;
extern func read(fd: int, buf: int, n: int): int;
extern func close(fd: int): int;
```

---

## Документация

Полная документация в виде книги расположена в папке `docs/`:

| Глава | Файл | Содержание |
|-------|------|-----------|
| 1 | [`docs/ch01_introduction.md`](docs/ch01_introduction.md) | Введение в Orbitron |
| 2 | [`docs/ch02_getting_started.md`](docs/ch02_getting_started.md) | Быстрый старт |
| 3 | [`docs/ch03_basics.md`](docs/ch03_basics.md) | Основы языка |
| 4 | [`docs/ch04_control_flow.md`](docs/ch04_control_flow.md) | Управление потоком |
| 5 | [`docs/ch05_functions.md`](docs/ch05_functions.md) | Функции |
| 6 | [`docs/ch06_collections.md`](docs/ch06_collections.md) | Коллекции: массивы и enum |
| 7 | [`docs/ch07_oop.md`](docs/ch07_oop.md) | ООП: структуры и классы |
| 8 | [`docs/ch08_features.md`](docs/ch08_features.md) | 10 особых возможностей |
| 9 | [`docs/ch09_stdlib.md`](docs/ch09_stdlib.md) | Стандартная библиотека |
| 10 | [`docs/ch10_projects.md`](docs/ch10_projects.md) | Проекты и модули |
| 11 | [`docs/ch11_lowlevel.md`](docs/ch11_lowlevel.md) | Низкоуровневое программирование |
| 12 | [`docs/ch12_backends.md`](docs/ch12_backends.md) | Бэкенды компиляции |
| — | [`docs/reference.md`](docs/reference.md) | Краткий справочник (шпаргалка) |
| — | [`docs/SUMMARY.md`](docs/SUMMARY.md) | Оглавление книги |

---

## Примеры

Примеры организованы по темам в пронумерованных папках:

### 01 — Основы
| Файл | Тема |
|------|------|
| [`examples/01_basics/hello.ot`](examples/01_basics/hello.ot) | Hello World, переменные, интерполяция |
| [`examples/01_basics/variables.ot`](examples/01_basics/variables.ot) | Типы, константы, операторы |
| [`examples/01_basics/operators.ot`](examples/01_basics/operators.ot) | Все операторы: арифм., лог., конвейер |
| [`examples/01_basics/input.ot`](examples/01_basics/input.ot) | Ввод с клавиатуры `readInt` / `readFloat` |

### 02 — Управление потоком
| Файл | Тема |
|------|------|
| [`examples/02_control_flow/conditionals.ot`](examples/02_control_flow/conditionals.ot) | if / else / unless / тернарный |
| [`examples/02_control_flow/loops.ot`](examples/02_control_flow/loops.ot) | Все виды циклов |
| [`examples/02_control_flow/match.ot`](examples/02_control_flow/match.ot) | match на числах и enum |

### 03 — Функции
| Файл | Тема |
|------|------|
| [`examples/03_functions/basics.ot`](examples/03_functions/basics.ot) | Объявление, параметры, рекурсия |
| [`examples/03_functions/recursion.ot`](examples/03_functions/recursion.ot) | Факториал, Фибоначчи, НОД, степень |
| [`examples/03_functions/pipe.ot`](examples/03_functions/pipe.ot) | Оператор конвейера `\|>` |

### 04 — Коллекции
| Файл | Тема |
|------|------|
| [`examples/04_collections/arrays.ot`](examples/04_collections/arrays.ot) | Массивы, сортировка, поиск |
| [`examples/04_collections/enums.ot`](examples/04_collections/enums.ot) | Enum: состояния, индексы, match |
| [`examples/04_collections/sorting.ot`](examples/04_collections/sorting.ot) | Пузырьковая, выборочная, вставка |

### 05 — ООП
| Файл | Тема |
|------|------|
| [`examples/05_oop/structs.ot`](examples/05_oop/structs.ot) | struct + impl (Vec2, Circle, Rect) |
| [`examples/05_oop/classes.ot`](examples/05_oop/classes.ot) | class + init (счёт, стек, матрица) |

### 06 — Стандартная библиотека
| Файл | Тема |
|------|------|
| [`examples/06_stdlib/math_demo.ot`](examples/06_stdlib/math_demo.ot) | Все функции `std/math` |
| [`examples/06_stdlib/bits_demo.ot`](examples/06_stdlib/bits_demo.ot) | Все функции `std/bits` |
| [`examples/06_stdlib/algo_demo.ot`](examples/06_stdlib/algo_demo.ot) | Все функции `std/algo` |

### 07 — Продвинутые темы
| Файл | Тема |
|------|------|
| [`examples/07_advanced/features.ot`](examples/07_advanced/features.ot) | Все 10 особых возможностей |
| [`examples/07_advanced/syscall_demo.ot`](examples/07_advanced/syscall_demo.ot) | Указатели и прямые системные вызовы |
| [`examples/07_advanced/net_demo.ot`](examples/07_advanced/net_demo.ot) | TCP/UDP сетевое программирование |

### 08 — Многофайловые проекты
| Проект | Описание |
|--------|---------|
| [`examples/08_projects/calculator/`](examples/08_projects/calculator/) | Калькулятор: main.ot + math.ot |
| [`examples/08_projects/geometry/`](examples/08_projects/geometry/) | Геометрия: main.ot + vectors.ot + shapes.ot |

---

## Конвейер компиляции

```
┌──────────────────────────────────────────────────────────────┐
│                        Бэкенд LLVM                           │
│                                                              │
│  source.ot  ─►  Lexer  ─►  Parser  ─►  Resolver             │
│                                             │                │
│                              Слитый AST ◄───┘                │
│                                  │                           │
│                              CodeGen  ─►  LLVM IR (.ll)      │
│                                               │              │
│                                  llc ◄────────┘              │
│                                   │                          │
│                              Ассемблер (.s)                  │
│                                   │                          │
│                              clang -lm                       │
│                                   │                          │
│                           Нативный бинарник                  │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│                        Бэкенд JVM                            │
│                                                              │
│  source.ot ─► Lexer ─► Parser ─► Resolver ─► JvmCodeGen     │
│                                                   │          │
│                             Main.java ◄───────────┘          │
│                                  │                           │
│                             javac + jar                      │
│                                  │                           │
│                            output.jar                        │
└──────────────────────────────────────────────────────────────┘
```

---

## Структура репозитория

```
src/
├── main.rs          Диспетчер CLI
├── cli.rs           Разбор аргументов, справка, enum Backend
├── pipeline.rs      Конвейеры компиляции LLVM и JVM
├── error.rs         Тип CompileError
├── project.rs       Чтение манифеста orbitron.toml
├── resolver.rs      Рекурсивный резолвер импортов и слияние AST
├── lexer/
│   ├── mod.rs       Лексер — токенизация .ot исходников
│   └── token.rs     Перечисление токенов и таблица ключевых слов
├── parser/
│   ├── mod.rs       Рекурсивный нисходящий парсер
│   └── ast.rs       Типы узлов AST (Expr, Stmt, ...)
├── codegen/
│   ├── mod.rs       Структура CodeGen, трёхпроходная кодогенерация
│   ├── expr.rs      Выражение → LLVM IR
│   └── stmt.rs      Инструкция → LLVM IR
└── jvm/
    └── mod.rs       AST → Java-исходник → javac → jar

stdlib/              Стандартная библиотека (на самом Orbitron)
docs/                Документация в виде книги (12 глав, по-русски)
examples/            Примеры программ, организованные по темам
```

---

## Лицензия

MIT — смотрите [LICENSE](LICENSE).
