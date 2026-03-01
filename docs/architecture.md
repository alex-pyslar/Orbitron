# Архитектура компилятора Orbitron

Orbitron компилирует исходный код `.ot` в нативный бинарный файл через LLVM IR.

## Пайплайн компиляции

```
исходный код .ot
      │
      ▼  Резолвер (src/resolver.rs)
   Объединённый AST (все файлы + импорты)
      │
      ▼  Лексер (src/lexer/)
   Поток токенов
      │
      ▼  Парсер (src/parser/)
   Абстрактное синтаксическое дерево
      │
      ▼  Кодогенератор (src/codegen/)
   LLVM IR (.ll)
      │
      ▼  llc
   Ассемблер (.s)
      │
      ▼  clang
   Нативный бинарник
```

> Резолвер вызывает лексер и парсер рекурсивно для каждого импортируемого файла,
> затем объединяет все AST в один плоский `Vec<Stmt>` в порядке зависимостей.

---

## Структура исходного кода

```
src/
├── main.rs          — точка входа CLI; команды new / build / run / <file.ot>
├── error.rs         — тип CompileError
├── project.rs       — ProjectManifest + load_manifest()
├── resolver.rs      — рекурсивный резолвер импортов (AST merger)
├── lexer/
│   ├── mod.rs       — struct Lexer, tokenize()
│   └── token.rs     — enum Token, таблица ключевых слов
├── parser/
│   ├── mod.rs       — struct Parser, parse_*() методы
│   └── ast.rs       — узлы AST: Expr, Stmt, BinOp, UnaryOp, ...
└── codegen/
    ├── mod.rs       — struct CodeGen, generate_program(), save_and_compile()
    ├── expr.rs      — gen_expr(), gen_binop(), приведение типов
    └── stmt.rs      — gen_stmt() — все виды операторов
```

---

## Точка входа — `src/main.rs`

CLI диспетчер: разбирает аргументы и вызывает одну из команд.

```
orbitron new <name>       → cmd_new()
orbitron build [opts]     → cmd_build_or_run(run=false)
orbitron run   [opts]     → cmd_build_or_run(run=true)
orbitron <file.ot> [opts] → cmd_file()
```

Общий пайплайн компиляции вынесен в `compile_entry(entry, src_root, output, opts)`:

```rust
fn compile_entry(entry, src_root, output, opts) -> Result<()> {
    // 1. Резолвер — обходит граф импортов, возвращает плоский AST
    let program = resolver::resolve(entry, src_root, &mut visited)?;

    // 2. Кодогенерация — AST → LLVM IR
    let ctx = Context::create();
    let mut cg = CodeGen::new("orbitron", &ctx);
    cg.generate_program(&program);

    // 3. Компиляция — LLVM IR → бинарник
    cg.save_and_compile(output, opts)?;
}
```

`cmd_build_or_run` ищет `orbitron.toml` вверх по дереву директорий от CWD,
что позволяет запускать `orbitron build` из любой поддиректории проекта.

---

## Система проектов — `src/project.rs`

Читает и десериализует `orbitron.toml`:

```rust
pub struct ProjectManifest {
    pub project: ProjectSection,   // name, version
    pub build:   BuildSection,     // main, output
}

pub fn load_manifest(root: &Path) -> Result<ProjectManifest, String>
```

Пример `orbitron.toml`:

```toml
[project]
name    = "myapp"
version = "0.1.0"

[build]
main   = "src/main.ot"
output = "bin/myapp"
```

---

## Резолвер импортов — `src/resolver.rs`

```rust
pub fn resolve(
    entry:    &Path,
    src_root: &Path,
    visited:  &mut HashSet<PathBuf>,
) -> Result<Vec<Stmt>, String>
```

**Алгоритм:**

1. Канонизировать путь (`fs::canonicalize`) для надёжной дедупликации.
2. Если путь уже в `visited` → вернуть пустой список (файл уже включён).
3. Добавить путь в `visited` *перед* рекурсией — обнаруживает циклические импорты.
4. Лексировать и парсировать файл.
5. Обойти AST:
   - `Stmt::Import { path }` → рекурсивно вызвать `resolve(src_root/path.ot, ...)`
   - Остальные операторы → добавить в результат напрямую
6. Вернуть объединённый `Vec<Stmt>` (зависимости — *перед* использующим кодом).

**Правило разрешения пути:**

```
import "math"      →  {src_root}/math.ot
import "net/http"  →  {src_root}/net/http.ot
```

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
- `$"текст {var} конец"` → `Token::InterpolStr(parts)`, где parts — строки и имена переменных
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
EnumVariant { enum_name: String, variant: String }
Input | InputFloat
SelfExpr
```

**Операторы (`Stmt`):**
```
Import { path: String }                   ← обрабатывается резолвером, кодогенератор игнорирует
FnDecl { name, params, body }
StructDecl { name, fields }
ImplDecl { struct_name, methods }
ClassDecl { name, fields, methods }
EnumDecl { name, variants }
ConstDecl { name, value }
Let { name, expr }                        ← var name = expr
Assign { name, expr }
FieldAssign { obj, field, val }
IndexAssign { arr, idx, val }
CompoundAssign { name, op, expr }         ← +=, -=, *=, /=
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

- `looks_like_struct_lit()` — отличает `Name { field: val }` от `match expr { ... }`:
  структурный литерал распознаётся только если после `{` идёт `ident :` или сразу `}`
- `for i in 0..3, j in 0..5` — многодиапазонный `for` десахаризируется в два вложенных `Stmt::For` прямо в парсере
- `init(params)` → `MethodDecl { name: "new", has_self: true }`
- `i += 1` → `Stmt::CompoundAssign { name: "i", op: Add, expr: Number(1) }`
- `|>` — десахаризируется в вызов функции при парсинге

---

## Кодогенератор — `src/codegen/`

Использует библиотеку `inkwell` — безопасные Rust-привязки к LLVM C API.

### `mod.rs` — структура `CodeGen`

```rust
pub struct CodeGen<'ctx> {
    ctx:           &'ctx Context,
    builder:       Builder<'ctx>,
    module:        Module<'ctx>,
    fn_val:        Option<FunctionValue<'ctx>>,

    vars:          HashMap<String, Var<'ctx>>,   // текущие переменные: имя → (ptr, is_float)
    i64_ty:        IntType<'ctx>,
    f64_ty:        FloatType<'ctx>,
    ptr_ty:        PointerType<'ctx>,

    // ООП
    struct_types:  HashMap<String, StructType<'ctx>>,
    struct_fields: HashMap<String, Vec<(String, bool)>>,

    // Управление потоком
    loop_stack:    Vec<(BasicBlock<'ctx>, BasicBlock<'ctx>)>,  // (continue_bb, break_bb)

    // Новые фичи
    consts:    HashMap<String, ConstVal>,
    enum_defs: HashMap<String, HashMap<String, i64>>,
    deferred:  Vec<Stmt>,
}
```

### Значения в кодогенераторе

```rust
enum Val<'ctx> {
    Int(IntValue<'ctx>),
    Float(FloatValue<'ctx>),
    Struct(PointerValue<'ctx>, String),   // ptr + имя типа
}
```

При смешанной арифметике `int + float` — оба приводятся к `f64`.

### Трёхпроходная генерация (`generate_program`)

| Проход | Что делает |
|--------|------------|
| **0**  | Объявить LLVM-типы для struct/class (`declare_struct`); собрать enum-варианты в `enum_defs`; константы в `consts` |
| **1**  | Предварительно объявить сигнатуры всех функций и методов — позволяет взаимную рекурсию |
| **2**  | Генерировать тела функций и методов |

### Именование методов в LLVM IR

```
Orbitron:  pub func tick(self) { ... }   // в class/impl Counter
LLVM:      define i64 @Counter_tick(ptr %0) { ... }

Orbitron:  init(v: int, s: int) { ... }
LLVM:      define void @Counter_new(ptr %0, i64 %1, i64 %2) { ... }
```

`self` — первый аргумент типа `ptr` (указатель на аллоцированную структуру).

### Доступ к полям структуры

```llvm
; self.x = self.x + 1
%gep = getelementptr %Vec2, ptr %self, i32 0, i32 <field_index>
%val = load i64, ptr %gep
%new = add nsw i64 %val, 1
store i64 %new, ptr %gep
```

### Реализация новых фич

| Фича       | Реализация в кодогенераторе |
|------------|-----------------------------|
| `const`    | `HashMap<String, ConstVal>` в `CodeGen`; проверяется до поиска в `vars` |
| `**`       | Вызов `pow()` из `libm`; приведение обратно к `i64` если оба операнда были `int` |
| `\|>`      | Десахаризация в парсере → обычный `Call` |
| `unless`   | Десахаризация в парсере → `If { cond: Unary(Not, ...) }` |
| `$"..."`   | Токен `InterpolStr(parts)` → `printf` с форматными строками |
| `[...]`    | `alloca [N x i64]` + `getelementptr` по вычисленному индексу |
| `enum`     | `HashMap<enum, HashMap<variant, i64>>` в `CodeGen`; `EnumVariant` → `Number(val)` |
| `defer`    | `Vec<Stmt>` в `CodeGen`; `emit_deferred()` перед каждым `return` и в конце функции |
| `repeat N` | Десахаризация в парсере → `Stmt::For` со скрытой переменной `__ri` |
| `? :`      | Phi-узел в LLVM (а не `select` — для корректной обработки побочных эффектов) |

### Генерация цикла `for..in`

```
for i in from..to                    for i in from..=to
─────────────────────────            ─────────────────────────
alloca i                             alloca i
store from → i                       store from → i
br for.cond                          br for.cond

for.cond:                            for.cond:
  load i                               load i
  icmp SLT i, to                       icmp SLE i, to
  br → for.body / exit                 br → for.body / exit

for.body:                            for.body:
  [тело]                               [тело]
  i += 1                               i += 1
  br for.cond                          br for.cond

exit:                                exit:
```

### Встроенные функции

При инициализации `CodeGen::new()` объявляются в модуле:

```c
int    printf(char*, ...);   // println
int    scanf(char*, ...);    // readInt / readFloat
double pow(double, double);  // оператор **
```

Маппинг `println`:

| Тип значения | Вызов                    |
|--------------|--------------------------|
| `int`        | `printf("%lld\n", v)`    |
| `float`      | `printf("%g\n", v)`      |
| `"string"`   | `printf("%s\n", ptr)`    |
| `$"..."`     | `printf("<fmt>\n", ...)` |

### `save_and_compile(output, opts)` — пайплайн

1. `module.print_to_file("<output>.ll")` — запись LLVM IR
2. Если `--emit-llvm` → остановиться здесь
3. `llc <output>.ll -o <output>.s -relocation-model=pic` — в ассемблер
4. `clang <output>.s -o <output> -lm` — линковка с libm
5. Если не `--save-temps` → удалить `.ll` и `.s`

---

## Опции компиляции (`CompileOptions`)

```rust
// src/codegen/mod.rs
pub struct CompileOptions {
    pub emit_llvm:  bool,   // --emit-llvm
    pub save_temps: bool,   // --save-temps
    pub verbose:    bool,   // -v / --verbose
}
```

---

## Расширение компилятора

### Добавить новый оператор / синтаксическую конструкцию

1. **`token.rs`** — добавить вариант в `Token`; распознать в `next_token()`
2. **`ast.rs`** — добавить вариант в `Stmt` или `Expr`
3. **`parser/mod.rs`** — распознать и построить узел AST
4. **`codegen/stmt.rs`** или **`codegen/expr.rs`** — генерировать LLVM IR

### Добавить новый тип данных

1. Добавить вариант в `Val` (`codegen/mod.rs`)
2. Расширить `declare_struct`, `gen_expr`, `gen_stmt`
3. Добавить LLVM-тип в `CodeGen::new()`

### Добавить статическую проверку типов (TypeChecker)

Проход между парсером и кодогенератором:

```
AST → TypeChecker → аннотированный AST → CodeGen
```

TypeChecker обходит AST и заполняет `HashMap<String, Type>`, затем передаёт
аннотированное дерево в кодогенератор.

---

## Зависимости (`Cargo.toml`)

| Крейт      | Назначение                         |
|------------|------------------------------------|
| `inkwell`  | Генерация LLVM IR (safe Rust API)  |
| `llvm-sys` | C-привязки к LLVM (транзитивная)   |
| `serde`    | Десериализация манифеста           |
| `toml`     | Парсинг TOML                       |

Версия LLVM: 18.x (управляется feature-флагом `inkwell`).
