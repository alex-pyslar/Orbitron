# Глава 2 — Быстрый старт

## Предварительные требования

Прежде чем строить и запускать программы на Orbitron, нужно установить несколько
инструментов. Конкретный набор зависит от того, какой бэкенд компиляции вы планируете использовать.

### Для бэкенда LLVM (нативные бинарники — по умолчанию)

| Инструмент | Версия | Назначение |
|-----------|--------|-----------|
| **Rust + Cargo** | 1.70+ | Сборка самого компилятора |
| **LLVM** | 18.x | `llc` — компиляция LLVM IR |
| **Clang** | 18.x | Линковка финального бинарника |
| **libm** | любая | Математическая библиотека (для оператора `**`) |

**Ubuntu / Debian:**

```bash
# Установить Rust
curl https://sh.rustup.rs -sSf | sh
source ~/.cargo/env

# Установить LLVM 18 через официальный репозиторий apt
wget -O /tmp/llvm.sh https://apt.llvm.org/llvm.sh
sudo bash /tmp/llvm.sh 18

# Установить clang и libm
sudo apt install clang-18 libc-dev
```

> На большинстве систем Ubuntu 22.04+ `libm` уже есть в составе `libc`.

### Для бэкенда JVM (файлы `.jar` — опционально)

| Инструмент | Версия | Назначение |
|-----------|--------|-----------|
| **JDK** | 11+ | Команды `javac` и `jar` |

```bash
sudo apt install default-jdk
```

Также можно использовать GraalVM для компиляции `.jar`-файлов в нативные бинарники.

---

## Установка

### Сборка из исходников

```bash
git clone https://github.com/alex-pyslar/Orbitron.git
cd Orbitron
cargo build --release
```

Бинарник компилятора окажется по пути `target/release/orbitron`.

### Добавление в PATH

```bash
# Вариант А — добавить для текущей сессии
export PATH="$PATH:$(pwd)/target/release"

# Вариант Б — символическая ссылка в системную директорию
sudo ln -s $(pwd)/target/release/orbitron /usr/local/bin/orbitron
```

### Стандартная библиотека

Папка `stdlib/` должна находиться в одном из мест:
1. **Рядом с бинарником `orbitron`** — компилятор найдёт её автоматически.
2. **По пути `$ORBITRON_HOME/stdlib/`** — задайте переменную окружения:

```bash
export ORBITRON_HOME=/path/to/Orbitron
```

### Windows (через WSL)

Orbitron работает через **Windows Subsystem for Linux**. Все команды ниже
используют WSL для доступа к Linux-окружению:

```bash
# Сборка компилятора
wsl -e bash -c "cd /mnt/c/source/Orbitron && cargo build --release 2>&1"

# Компиляция и запуск одного файла
wsl -e bash -c "cd /mnt/c/source/Orbitron && \
    ./target/release/orbitron examples/01_basics/hello.ot && ./hello"
```

---

## Проверка установки

```bash
orbitron --help
```

Ожидаемый вывод:

```
USAGE:
  orbitron new <name>            Создать новый проект
  orbitron build [options]       Собрать проект (читает orbitron.toml)
  orbitron run   [options]       Собрать и запустить проект
  orbitron <file.ot> [options]   Скомпилировать один файл

OPTIONS:
  -h, --help              Показать справку и выйти
      --version           Показать версию и выйти
  -o <file>               Имя выходного файла
      --backend llvm|jvm  Бэкенд компиляции (по умолчанию: llvm)
      --emit-llvm         Сохранить LLVM IR (.ll) и остановиться
      --emit-java         Сохранить Java-исходник (.java) и остановиться
      --save-temps        Сохранить промежуточные файлы (.ll, .s)
  -v, --verbose           Выводить каждый шаг компиляции
```

---

## Первая программа

### Шаг 1 — Создайте файл

Создайте файл `hello.ot`:

```orbitron
fn main() {
    println("Привет, мир!");
    var версия = 1;
    println($"Добро пожаловать в Orbitron v{версия}!");
}
```

> Имена переменных должны быть на ASCII (латиница). Строки в `println()` могут
> содержать любой текст, в том числе кириллицу.

### Шаг 2 — Скомпилируйте

```bash
orbitron hello.ot
```

Эта команда создаёт нативный бинарник `hello` в текущей директории.

### Шаг 3 — Запустите

```bash
./hello
```

Вывод:

```
Привет, мир!
Добро пожаловать в Orbitron v1!
```

### Что произошло?

```
hello.ot
   │
   ▼  Лексер (разбивает исходник на токены)
   │
   ▼  Парсер (строит AST — дерево разбора)
   │
   ▼  Резолвер (обрабатывает импорты)
   │
   ▼  Генератор кода (создаёт LLVM IR → hello.ll)
   │
   ▼  llc (компилирует в ассемблер → hello.s)
   │
   ▼  clang -lm (линкует в нативный бинарник → hello)
```

Используйте `--verbose` или `--emit-llvm`, чтобы заглянуть внутрь:

```bash
orbitron hello.ot --verbose    # выводит каждый шаг
orbitron hello.ot --emit-llvm  # останавливается на hello.ll, показывает IR
```

---

## Режим проекта

Для программ с несколькими файлами в Orbitron используется **проект** —
директория с файлом манифеста и структурированным деревом исходников.

### Создание проекта

```bash
orbitron new myapp
```

Создаётся следующая структура:

```
myapp/
├── orbitron.toml       ← манифест проекта
├── src/
│   └── main.ot         ← точка входа
└── bin/                ← сюда попадает скомпилированный бинарник
```

### Файл манифеста

`orbitron.toml`:

```toml
[project]
name    = "myapp"
version = "0.1.0"

[build]
main    = "src/main.ot"    # точка входа
output  = "bin/myapp"      # путь к выходному файлу
backend = "llvm"           # "llvm" или "jvm"
```

### Сборка и запуск

```bash
cd myapp

orbitron build         # компиляция → bin/myapp
orbitron run           # компиляция + запуск
```

Команда `run` для бэкенда LLVM запускает `bin/myapp` напрямую.
Для бэкенда JVM — запускает `java -jar bin/myapp.jar`.

### Добавление модулей

Создайте файл `src/math.ot`:

```orbitron
fn square(n: int): int {
    return n * n;
}

fn cube(n: int): int {
    return n * n * n;
}
```

Импортируйте его в `src/main.ot`:

```orbitron
import "math";   // загружает src/math.ot

fn main() {
    println(square(5));   // 25
    println(cube(3));     // 27
}
```

Подробное руководство по системе импортов — в [Главе 10](ch10_projects.md).

---

## Справочник по CLI

### Команды

| Команда | Описание |
|---------|----------|
| `orbitron new <name>` | Создать заготовку нового проекта |
| `orbitron build` | Скомпилировать текущий проект |
| `orbitron run` | Скомпилировать и запустить текущий проект |
| `orbitron <file.ot>` | Скомпилировать один файл |

### Флаги

| Флаг | Описание |
|------|----------|
| `-o <file>` | Переопределить имя выходного файла |
| `--backend llvm\|jvm` | Выбрать бэкенд компиляции |
| `--emit-llvm` | Сохранить LLVM IR (`.ll`) и остановиться |
| `--emit-java` | Сохранить Java-исходник (`.java`) и остановиться |
| `--save-temps` | Сохранить промежуточные файлы `.ll` и `.s` |
| `-v`, `--verbose` | Выводить каждый шаг компиляции |
| `-h`, `--help` | Показать справку |
| `--version` | Показать версию |

### Примеры

```bash
# Компиляция одного файла
orbitron hello.ot                  # → ./hello
orbitron hello.ot -o myprogram     # → ./myprogram
orbitron hello.ot --backend jvm    # → hello.jar
orbitron hello.ot --emit-llvm      # → hello.ll (без линковки)
orbitron hello.ot -v               # verbose: показывает каждый шаг

# Режим проекта
orbitron new calculator
cd calculator
orbitron build                     # → bin/calculator
orbitron run                       # сборка + запуск
orbitron build -o bin/calc_debug   # своё имя файла
orbitron build --backend jvm       # → bin/calculator.jar
orbitron run   --backend jvm       # сборка + java -jar
```

---

## Быстрый старт с бэкендом JVM

```bash
# Компиляция в .jar
orbitron hello.ot --backend jvm

# Запуск jar-файла
java -jar hello.jar

# Или через orbitron run
orbitron run --backend jvm

# Компиляция в нативный бинарник через GraalVM
native-image -jar hello.jar -o hello_native
```

Чтобы сделать JVM бэкендом по умолчанию для проекта, добавьте в `orbitron.toml`:

```toml
[build]
backend = "jvm"
```

---

## Первая настоящая программа

Напишем что-нибудь поинтереснее — программу для вычисления чисел Фибоначчи:

```orbitron
fn fib(n: int): int {
    if (n <= 1) { return n; }
    return fib(n - 1) + fib(n - 2);
}

fn main() {
    println("Последовательность Фибоначчи:");
    for i in 0..=10 {
        println(fib(i));
    }
}
```

Сохраните как `fib.ot`, скомпилируйте и запустите:

```bash
orbitron fib.ot && ./fib
```

Вывод:

```
Последовательность Фибоначчи:
0
1
1
2
3
5
8
13
21
34
55
```

---

## Что дальше

Теперь, когда Orbitron установлен и вы знаете, как компилировать программы,
изучим сам язык:

| Тема | Глава |
|------|-------|
| Переменные, типы, операторы | [Глава 3 — Основы языка](ch03_basics.md) |
| if, циклы, match | [Глава 4 — Управление потоком](ch04_control_flow.md) |
| Функции и оператор конвейера | [Глава 5 — Функции](ch05_functions.md) |
| Массивы и перечисления | [Глава 6 — Коллекции](ch06_collections.md) |
| Структуры и классы | [Глава 7 — ООП](ch07_oop.md) |
| Все 10 специальных возможностей | [Глава 8 — Специальные возможности](ch08_features.md) |

---

← [Глава 1 — Введение](ch01_introduction.md) | [Глава 3 — Основы языка →](ch03_basics.md)
