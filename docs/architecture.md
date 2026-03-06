# Архитектура компилятора Orbitron

Orbitron компилирует исходный код `.ot` в нативный бинарный файл (через LLVM IR)
или в JVM-байткод (через Java исходники → javac → .jar).

## Пайплайн компиляции

### LLVM-бэкенд

```
исходный код .ot
      │
      ▼  Резолвер (src/resolver.rs)
   Объединённый AST (все файлы + stdlib)
      │
      ▼  Кодогенератор (src/codegen/)
   LLVM IR (.ll)
      │
      ▼  llc
   Ассемблер (.s)
      │
      ▼  clang -lm
   Нативный бинарник
```

### JVM-бэкенд

```
исходный код .ot
      │
      ▼  Резолвер (src/resolver.rs)
   Объединённый AST (все файлы + stdlib)
      │
      ▼  JVM кодогенератор (src/jvm/mod.rs)
   Main.java
      │
      ▼  javac
   .class файлы
      │
      ▼  jar cfm
   <output>.jar
```

---

## Структура исходного кода

```
src/
├── main.rs          — диспетчер команд: new / build / run / <file.ot>
├── cli.rs           — Backend, BuildOpts, print_help(), parse_build_opts()
├── pipeline.rs      — compile_llvm(), compile_jvm(), find_stdlib(), find_project_root()
├── error.rs         — тип CompileError
├── project.rs       — ProjectManifest + load_manifest()
├── resolver.rs      — рекурсивный резолвер импортов (AST merger)
├── lexer/
│   ├── mod.rs       — struct Lexer, tokenize()
│   └── token.rs     — enum Token, таблица ключевых слов
├── parser/
│   ├── mod.rs       — struct Parser, parse_*() методы
│   └── ast.rs       — узлы AST: Expr, Stmt, BinOp, UnaryOp, ...
├── codegen/
│   ├── mod.rs       — struct CodeGen, generate_program(), save_and_compile()
│   ├── expr.rs      — gen_expr(), gen_binop(), приведение типов
│   └── stmt.rs      — gen_stmt() — все виды операторов
└── jvm/
    └── mod.rs       — JvmCodeGen, generate_and_compile()

stdlib/
├── math.ot          — математические функции (import "std/math")
├── bits.ot          — битовые операции (import "std/bits")
└── algo.ot          — вспомогательные алгоритмы (import "std/algo")
```

---

## Точка входа — `src/main.rs`

CLI диспетчер: разбирает первый аргумент и вызывает одну из команд.

```
orbitron new <name>       → cmd_new()
orbitron build [opts]     → cmd_build_or_run(run=false)
orbitron run   [opts]     → cmd_build_or_run(run=true)
orbitron <file.ot> [opts] → cmd_file()
```

`cmd_build_or_run` ищет `orbitron.toml` вверх по дереву директорий от CWD.

---

## CLI — `src/cli.rs`

Содержит всё, связанное с разбором аргументов командной строки:

```rust
pub enum Backend { Llvm, Jvm }

pub struct BuildOpts {
    pub output:     Option<String>,
    pub backend:    Option<Backend>, // CLI override
    pub emit_llvm:  bool,
    pub emit_java:  bool,
    pub save_temps: bool,
    pub verbose:    bool,
}

pub fn print_help()
pub fn parse_build_opts(args: &[String]) -> Result<BuildOpts, String>
```

---

## Пайплайны — `src/pipeline.rs`

```rust
// Обнаружение stdlib ($ORBITRON_HOME/stdlib/ или {exe_dir}/stdlib/)
pub fn find_stdlib() -> Option<PathBuf>

// Поиск orbitron.toml вверх по дереву
pub fn find_project_root(start: &Path) -> Option<PathBuf>

// Полный LLVM-пайплайн: resolver → codegen → llc → clang
pub fn compile_llvm(entry, src_root, output, opts) -> Result<(), CompileError>

// Полный JVM-пайплайн: resolver → jvm codegen → javac → jar
pub fn compile_jvm(entry, src_root, output, opts) -> Result<(), String>
```

---

## Система проектов — `src/project.rs`

Читает и десериализует `orbitron.toml`:

```rust
pub struct ProjectManifest {
    pub project: ProjectSection,   // name, version
    pub build:   BuildSection,     // main, output, backend
}

pub fn load_manifest(root: &Path) -> Result<ProjectManifest, String>
```

Пример `orbitron.toml`:

```toml
[project]
name    = "myapp"
version = "0.1.0"

[build]
main    = "src/main.ot"
output  = "bin/myapp"
backend = "llvm"    # или "jvm"
```

---

## Резолвер импортов — `src/resolver.rs`

```rust
pub fn resolve(
    entry:       &Path,
    src_root:    &Path,
    stdlib_root: Option<&Path>,
    visited:     &mut HashSet<PathBuf>,
) -> Result<Vec<Stmt>, String>
```

**Алгоритм:**

1. Канонизировать путь (`fs::canonicalize`) для надёжной дедупликации.
2. Если путь уже в `visited` → вернуть пустой список.
3. Добавить путь в `visited` *перед* рекурсией — обнаруживает циклы.
4. Лексировать и парсировать файл.
5. Обойти AST: `Import` → рекурсия; остальное → добавить в результат.
6. Вернуть объединённый `Vec<Stmt>` (зависимости *перед* использующим кодом).

**Правило разрешения пути:**

```
import "math"       →  {src_root}/math.ot
import "net/http"   →  {src_root}/net/http.ot
import "std/math"   →  {stdlib_root}/math.ot
import "std/bits"   →  {stdlib_root}/bits.ot
import "std/algo"   →  {stdlib_root}/algo.ot
```

**Поиск stdlib** (в `src/pipeline.rs::find_stdlib()`):
1. `$ORBITRON_HOME/stdlib/`
2. `{exe_dir}/stdlib/` — папка рядом с бинарником

---

## Лексер — `src/lexer/`

### `token.rs` — перечисление `Token`

```rust
pub enum Token {
    // Литералы
    Int(i64), Float(f64), Str(String),
    InterpolStr(Vec<InterpolPart>),   // $"...{var}..."

    // Ключевые слова
    Func, Var, Const, Return, If, Else, Unless,
    While, Do, For, In, Loop, Break, Continue,
    Repeat, Match, Struct, Impl, Class, Init, New,
    Pub, Private, SelfKw, True, False,
    Enum, Defer, Import,

    // Идентификаторы
    Ident(String), Println,

    // Операторы
    Plus, Minus, Star, Slash, Percent, StarStar,
    PlusAssign, MinusAssign, StarAssign, SlashAssign,
    Assign, EqEq, BangEq, Lt, LtEq, Gt, GtEq,
    AndAnd, OrOr, Bang, PipeGt,

    // Пунктуация
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Semicolon, Colon, Comma, Dot, FatArrow, DotDot, DotDotEq,

    Eof,
}
```

**Особенности лексера:**
- `1..5` не путается с `1.5`: после `.` проверяется, что следующий символ — тоже `.`
- `$"текст {var} конец"` → `Token::InterpolStr(parts)`
- Комментарии `//` (до конца строки) и `/* */` (многострочные) пропускаются
- `CRLF` → `LF` нормализуется перед лексированием

---

## Парсер — `src/parser/`

Рекурсивный нисходящий парсер (Recursive Descent).

### `ast.rs` — узлы AST

**Выражения (`Expr`):**
```
Number(i64)  |  Float(f64)  |  Str(String)  |  Ident(String)
Binary(Box<Expr>, BinOp, Box<Expr>)
Unary(UnaryOp, Box<Expr>)
Call { name: String, args: Vec<Expr> }
MethodCall { obj: Box<Expr>, method: String, args: Vec<Expr> }
FieldAccess { obj: Box<Expr>, field: String }
StructLit { name: String, fields: Vec<(String, Expr)> }
ConstructorCall { class: String, args: Vec<Expr> }
ArrayLit(Vec<Expr>)
Index { arr: Box<Expr>, idx: Box<Expr> }
Input | InputFloat
SelfExpr
```

**Операторы (`Stmt`):**
```
Import { path: String }           ← обрабатывается резолвером, кодогенератор игнорирует
FnDecl { name, params, body }
StructDecl { name, fields }
ImplDecl { struct_name, methods }
ClassDecl { name, fields, methods }
EnumDecl { name, variants }
Const { name, expr }
Let { name, expr }                ← var name = expr
Assign { name, expr }
FieldAssign { obj, field, val }
IndexAssign { arr, idx, val }
CompoundAssign { name, op, expr } ← +=, -=, *=, /=
Return(Expr) | Print(Expr)
If { cond, then, els }
While { cond, body }
DoWhile { body, cond }
For { var, from, to, inclusive, body }
Loop { body }
Break | Continue
Repeat { count, body }
Match { expr, arms }
Defer(Box<Stmt>)
Block(Vec<Stmt>) | Expr(Expr)
```

### `mod.rs` — иерархия приоритетов (от низкого к высокому)

```
parse_pipe      |>
parse_ternary   ? :
parse_or        ||
parse_and       &&
parse_cmp       == != < <= > >=
parse_add       + -
parse_mul       * / %
parse_unary     - !
parse_power     **      (право-ассоциативный)
parse_postfix   .field  .method(args)  [idx]
parse_call_base new Name(args) | Name(args) | Name{...} | [arr] | readInt/Float | primary
parse_primary   литерал | ident | self | (expr)
```

**Трудные места парсера:**

- `looks_like_struct_lit()` — отличает `Name { field: val }` от `match expr { ... }`
- `for i in 0..3, j in 0..5` — многодиапазонный for → два вложенных `Stmt::For`
- `init(params)` → `MethodDecl { name: "new", has_self: true }`
- `i += 1` → `Stmt::CompoundAssign { name: "i", op: Add, expr: Number(1) }`
- `|>` — десахаризируется в вызов функции при парсинге

---

## LLVM Кодогенератор — `src/codegen/`

Использует библиотеку `inkwell` — безопасные Rust-привязки к LLVM C API.

### `mod.rs` — структура `CodeGen`

```rust
pub struct CodeGen<'ctx> {
    ctx:           &'ctx Context,
    builder:       Builder<'ctx>,
    module:        Module<'ctx>,

    vars:          HashMap<String, Var<'ctx>>,   // имя → (ptr, kind)
    i64_ty:        IntType<'ctx>,
    f64_ty:        FloatType<'ctx>,

    struct_types:  HashMap<String, StructType<'ctx>>,
    struct_fields: HashMap<String, Vec<(String, bool)>>,

    loop_stack:    Vec<(BasicBlock<'ctx>, BasicBlock<'ctx>)>,

    consts:   HashMap<String, ConstVal>,
    enums:    HashMap<String, HashMap<String, i64>>,
    deferred: Vec<Stmt>,
}
```

### Трёхпроходная генерация (`generate_program`)

| Проход | Что делает |
|--------|------------|
| **0**  | Объявить LLVM-типы для struct/class; собрать enum-варианты; константы |
| **1**  | Предварительно объявить сигнатуры всех функций и методов |
| **2**  | Генерировать тела функций и методов |

### Именование методов в LLVM IR

```
Orbitron:  pub func tick(self)      // в class/impl Counter
LLVM:      define i64 @Counter_tick(ptr %0)

Orbitron:  init(v: int, s: int)
LLVM:      define i64 @Counter_new(ptr %0, i64 %1, i64 %2)
```

### Реализация языковых фич

| Фича       | Реализация |
|------------|------------|
| `const`    | `HashMap<String, ConstVal>` — проверяется до поиска в `vars` |
| `**`       | Вызов `pow()` из libm; приведение к i64 если оба аргумента int |
| `\|>`      | Десахаризация в парсере → обычный `Call` |
| `unless`   | Десахаризация в парсере → `If { cond: Unary(Not, ...) }` |
| `$"..."`   | `Token::InterpolStr(parts)` → `printf` с форматными строками |
| `[...]`    | `alloca [N x i64]` + `getelementptr` по вычисленному индексу |
| `enum`     | `HashMap<enum, HashMap<variant, i64>>` в CodeGen |
| `defer`    | `Vec<Stmt>` — `emit_deferred()` перед каждым return и в конце функции |
| `repeat N` | Десахаризация в парсере → `Stmt::For` со скрытой переменной `__ri` |
| `? :`      | Phi-узел в LLVM (не `select` — для корректных побочных эффектов) |

### Пайплайн `save_and_compile(output, opts)`

1. `module.print_to_file("<output>.ll")` — запись LLVM IR
2. Если `--emit-llvm` → остановиться
3. `llc <output>.ll -o <output>.s -relocation-model=pic`
4. `clang <output>.s -o <output> -lm`
5. Если не `--save-temps` → удалить `.ll` и `.s`

---

## JVM Кодогенератор — `src/jvm/mod.rs`

Транспилирует AST в Java исходник `Main.java`, затем вызывает `javac` и `jar`.

```rust
pub struct JvmOptions {
    pub emit_java: bool,   // --emit-java: остановиться после генерации .java
    pub verbose:   bool,
}

pub fn generate_and_compile(program: &[Stmt], output: &str, opts: &JvmOptions)
    -> Result<(), String>
```

**Маппинг типов Orbitron → Java:**

| Orbitron | Java       |
|----------|------------|
| `int`    | `long`     |
| `float`  | `double`   |
| массив   | `long[]`   |
| struct   | `static class` |
| `self`   | `this`     |
| `init`   | Java-конструктор |

**Особенности:**
- Enums: `EnumName_Variant` → статические `long` константы
- `match`: уникальные переменные `__m0`, `__m1`, ...
- `defer`: try-finally блок (LIFO порядок)
- Запуск: `java -jar <output>.jar`
- GraalVM: `native-image -jar <output>.jar`

---

## Стандартная библиотека — `stdlib/`

Написана на самом языке Orbitron — никаких специальных привилегий нет.

| Файл | Импорт | Содержимое |
|------|--------|------------|
| `math.ot` | `import "std/math"` | abs, max, min, clamp, factorial, fib, gcd, lcm, sum_range, sign, is_prime; константы PI, E, INT_MAX |
| `bits.ot` | `import "std/bits"` | bit_count, bit_len, is_pow2, next_pow2, prev_pow2, low_bit, shl, shr, floor_log2, reverse_bits |
| `algo.ot` | `import "std/algo"` | min3, max3, median3, lerp, map_range, dist, digit_count, digit_sum, reverse_digits, is_palindrome_num, ipow, triangle, is_triangle, isqrt, is_square, near, cycle |

---

## Встроенные функции (объявляются в `CodeGen::new()`)

```c
int    printf(char*, ...);   // println
int    scanf(char*, ...);    // readInt / readFloat
double pow(double, double);  // оператор **
```

---

## Зависимости (`Cargo.toml`)

| Крейт      | Назначение                         |
|------------|------------------------------------|
| `inkwell`  | Генерация LLVM IR (safe Rust API)  |
| `llvm-sys` | C-привязки к LLVM (транзитивная)   |
| `serde`    | Десериализация манифеста           |
| `toml`     | Парсинг TOML                       |

Версия LLVM: 18.x (управляется feature-флагом `inkwell`).

---

## Расширение компилятора

### Добавить новый оператор / синтаксическую конструкцию

1. **`token.rs`** — добавить вариант в `Token`; распознать в `next_token()`
2. **`ast.rs`** — добавить вариант в `Stmt` или `Expr`
3. **`parser/mod.rs`** — распознать и построить узел AST
4. **`codegen/stmt.rs`** или **`codegen/expr.rs`** — генерировать LLVM IR
5. **`jvm/mod.rs`** — генерировать Java (если нужен JVM-бэкенд)

### Добавить модуль в stdlib

Создать файл `stdlib/<name>.ot` с обычными Orbitron-функциями и константами.
Пользователь подключает его через `import "std/<name>"`.

### Добавить статическую проверку типов (TypeChecker)

Проход между парсером и кодогенератором:

```
AST → TypeChecker → аннотированный AST → CodeGen
```
