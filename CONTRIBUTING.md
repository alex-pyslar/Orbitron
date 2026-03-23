# Участие в разработке Orbitron

Спасибо за интерес к проекту! Этот документ объясняет, как собрать компилятор локально, где живёт каждая часть функциональности и какие соглашения приняты в кодовой базе.

---

## Содержание

1. [Требования](#требования)
2. [Сборка из исходников](#сборка-из-исходников)
3. [Запуск тестов](#запуск-тестов)
4. [Структура проекта](#структура-проекта)
5. [Как добавить возможность языка](#как-добавить-возможность-языка)
6. [Как добавить модуль стандартной библиотеки](#как-добавить-модуль-стандартной-библиотеки)
7. [Соглашения кодирования](#соглашения-кодирования)
8. [Отправка Pull Request](#отправка-pull-request)
9. [Сообщение об ошибках](#сообщение-об-ошибках)

---

## Требования

| Инструмент | Версия | Примечание |
|---|---|---|
| Rust + Cargo | 1.70+ | Установка через [rustup.rs](https://rustup.rs) |
| LLVM | 18.x | `sudo apt install llvm-18 clang-18` (Ubuntu/Debian) |
| JDK | 11+ | Необязательно — только для JVM-бэкенда |

На Ubuntu 22.04+ LLVM 18 устанавливается через официальный apt-репозиторий:

```bash
wget -O /tmp/llvm.sh https://apt.llvm.org/llvm.sh && sudo bash /tmp/llvm.sh 18
```

На Windows — используйте WSL2 с Ubuntu 22.04+ для полного тулчейна.

---

## Сборка из исходников

```bash
git clone https://github.com/alex-pyslar/Orbitron.git
cd Orbitron
cargo build --release
```

Бинарник компилятора: `target/release/orbitron`.

### Проверка сборки

```bash
./target/release/orbitron examples/01_basics/hello.ot -o /tmp/hello
/tmp/hello
# → Hello, World!
```

---

## Запуск тестов

Автоматизированного тестового набора пока нет — это отличная область для вклада!

Рекомендуемый рабочий процесс:

```bash
# 1. Сборка
cargo build --release 2>&1

# 2. Компиляция всех примеров (LLVM)
for f in examples/01_basics/*.ot examples/02_control_flow/*.ot examples/03_functions/*.ot; do
    echo "--- $f ---"
    ./target/release/orbitron "$f" -o /tmp/orb_test && /tmp/orb_test
done

# 3. Проверка JVM-бэкенда
./target/release/orbitron examples/01_basics/hello.ot --backend jvm -o /tmp/hello
java -jar /tmp/hello.jar
```

Если добавляете новую возможность — добавьте или обновите пример в `examples/`, чтобы проверяющие могли быстро убедиться в корректности.

---

## Структура проекта

```
src/
├── main.rs          Диспетчер CLI (new / build / run / fmt / <file.ot>)
├── cli.rs           BuildOpts, enum Backend, parse_build_opts()
├── pipeline.rs      compile_llvm(), compile_jvm(), find_stdlib()
├── error.rs         Тип CompileError
├── project.rs       Манифест orbitron.toml (serde + toml)
├── resolver.rs      Рекурсивный резолвер импортов (слияние AST)
├── lexer/
│   ├── mod.rs       struct Lexer, tokenize()
│   └── token.rs     enum Token, таблица ключевых слов
├── parser/
│   ├── mod.rs       Рекурсивный нисходящий парсер
│   └── ast.rs       Expr, Stmt, BinOp, UnaryOp, Access, ...
├── codegen/
│   ├── mod.rs       CodeGen, generate_program(), трёхпроходная кодогенерация
│   ├── expr.rs      gen_expr(), gen_binop()
│   └── stmt.rs      gen_stmt() для каждого варианта Stmt
├── jvm/
│   └── mod.rs       JvmCodeGen, generate_and_compile()
└── fmt/
    └── mod.rs       AST pretty-printer (команда orbitron fmt)

stdlib/              Стандартная библиотека на самом Orbitron
├── math.ot
├── bits.ot
├── algo.ot
├── sys.ot
├── net.ot
└── db.ot

examples/            Примеры с аннотациями (.ot)
docs/                Документация в виде книги (Markdown, по-русски)
```

---

## Как добавить возможность языка

Добавление нового оператора или синтаксической конструкции затрагивает до пяти файлов. Следуйте этим шагам по порядку:

### 1. `src/lexer/token.rs` — добавить вариант Token

```rust
pub enum Token {
    // ... существующие токены ...
    MyNewOp,   // добавьте свой вариант здесь
}
```

Распознать в `Lexer::next_token()` в `src/lexer/mod.rs`:

```rust
'@' => { self.advance(); Token::MyNewOp }
```

Ключевые слова добавляются в таблицу `match word { ... }` в том же файле:

```rust
"mynew" => Token::MyNewOp,
```

### 2. `src/parser/ast.rs` — добавить узел AST

Для нового выражения:

```rust
pub enum Expr {
    // ...
    MyNew(Box<Expr>),
}
```

Для новой инструкции:

```rust
pub enum Stmt {
    // ...
    MyNewStmt { expr: Expr },
}
```

### 3. `src/parser/mod.rs` — разобрать новую конструкцию

Вставьте вызов разбора на нужном уровне приоритета (смотрите цепочку `parse_pipe` → `parse_ternary` → ... → `parse_primary`).

### 4. `src/codegen/expr.rs` или `src/codegen/stmt.rs` — эмитировать LLVM IR

```rust
Expr::MyNew(inner) => {
    let val = self.gen_expr(inner, func, bb)?;
    // ... эмит LLVM IR ...
    Ok(val)
}
```

### 5. `src/jvm/mod.rs` — эмитировать Java (при необходимости)

Если возможность должна работать с `--backend jvm`, добавьте соответствующий arm в JVM кодогенератор. Если только LLVM, добавьте `panic!` с понятным сообщением:

```rust
Expr::MyNew(_) => panic!("MyNew не поддерживается в JVM-бэкенде"),
```

---

## Как добавить модуль стандартной библиотеки

Модули стандартной библиотеки — это обычные исходники на Orbitron.

1. Создайте `stdlib/<имя>.ot` с функциями и константами.
2. Пользователи импортируют через `import "std/<имя>";`.
3. Задокументируйте в `docs/ch09_stdlib.md`.
4. Добавьте небольшое демо в `examples/06_stdlib/`, если модуль нетривиален.

**Важные ограничения:**

- Не определяйте функцию с именем `pow` — она конфликтует с заранее объявленной libm `pow(double, double)`, используемой оператором `**`.
- Все параметры и возвращаемые значения — `i64`. Поддержки типизированных float-параметров в stdlib пока нет.
- Массивы пока нельзя передавать в функции или возвращать из них.

**Пример модуля stdlib:**

```orbitron
// stdlib/mymodule.ot

const MY_CONST: i64 = 42;

fn my_func(x: i64) -> i64 {
    return x * MY_CONST;
}
```

---

## Соглашения кодирования

**Rust:**
- Редакция: 2021
- Именование: `snake_case` для функций и переменных, `CamelCase` для типов
- Видимые пользователю строки (ошибки, verbose-вывод) — на английском
- При предупреждениях `dead_code` под rustc 1.93+ добавляйте `#![allow(dead_code)]` в `main.rs`

**Файлы Orbitron (`.ot`):**
- Только ASCII-идентификаторы (кириллица не поддерживается лексером)
- Комментарии в stdlib и примерах — на английском
- Отступы — 4 пробела (по правилам `orbitron fmt`)
- `{` — на той же строке, что и объявление

**Синтаксис `.ot` файлов — текущий стандарт:**

```orbitron
// ✓ правильно
var x = 5;
var mut count = 0;
const MAX: i64 = 100;
fn add(a: i64, b: i64) -> i64 { return a + b; }
public fn method(self) -> i64 { ... }
private var field: i64,

// ✗ устаревший синтаксис (не использовать)
// let x = 5;
// func add(...) { }
// pub fn method() { }
```

---

## Отправка Pull Request

1. Сделайте форк репозитория и создайте ветку:

   ```bash
   git checkout -b feature/my-feature
   ```

2. Внесите изменения и убедитесь, что все существующие примеры компилируются и дают правильный результат (см. [Запуск тестов](#запуск-тестов)).

3. Добавьте или обновите пример либо раздел документации, описывающий новое поведение.

4. Сделайте коммит с описательным сообщением:

   ```bash
   git commit -m "feat: repeat-N loop desugared to Stmt::For"
   ```

5. Откройте Pull Request в `main`. Опишите **что** изменилось и **почему**.

---

## Сообщение об ошибках

Откройте issue на GitHub и укажите:

- Исходный `.ot` файл, воспроизводящий ошибку (или минимальный пример).
- Точную команду запуска (`orbitron <file.ot> --backend llvm`, и т.д.).
- Полное сообщение об ошибке или неожиданный вывод.
- ОС, версию Rust (`rustc --version`) и версию LLVM (`llc --version`).
