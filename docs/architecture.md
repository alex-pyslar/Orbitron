# Архитектура компилятора Orbitron

Orbitron — это компилятор с исходника `.ot` в нативный бинарный файл через LLVM IR.

```
исходный код .ot
       │
       ▼ Лексер (src/lexer/)
    Поток токенов
       │
       ▼ Парсер (src/parser/)
    Абстрактное синтаксическое дерево (AST)
       │
       ▼ Кодогенерация (src/codegen/)
    LLVM IR (output.ll)
       │
       ▼ llc (LLVM backend)
    Ассемблер (output.s)
       │
       ▼ clang (линковка)
    Нативный бинарный файл
```

---

## Структура проекта

```
src/
├── main.rs              — точка входа, оркестрация пайплайна
├── error.rs             — тип CompileError
├── lexer/
│   ├── mod.rs           — структура Lexer, tokenize()
│   └── token.rs         — перечисление Token, таблица ключевых слов
├── parser/
│   ├── mod.rs           — структура Parser, все parse_*() методы
│   └── ast.rs           — узлы AST: Expr, Stmt, BinOp, UnaryOp, ...
└── codegen/
    ├── mod.rs           — структура CodeGen, generate_program(), save_and_compile()
    ├── expr.rs          — gen_expr(), gen_binop(), приведение типов
    └── stmt.rs          — gen_stmt() — все виды операторов
```

---

## Лексер (`src/lexer/`)

### `token.rs` — перечисление `Token`

Каждый лексический элемент программы представлен значением `Token`:

```rust
pub enum Token {
    // Литералы
    Int(i64), Float(f64), Str(String),
    // Ключевые слова
    Func, Var, Return, If, Else, While, Do, For, In, Loop,
    Break, Continue, Match, Struct, Impl, Class, Init,
    Pub, Private, New, SelfKw, True, False,
    // Идентификаторы и специальные токены
    Ident(String), Println,
    // Операторы
    Plus, Minus, Star, Slash, Percent,
    PlusAssign, MinusAssign, StarAssign, SlashAssign,
    Assign, EqEq, BangEq, Lt, LtEq, Gt, GtEq,
    AndAnd, OrOr, Bang,
    // Пунктуация
    LParen, RParen, LBrace, RBrace, Semicolon, Colon, Comma, Dot,
    FatArrow, DotDot, DotDotEq,
    // Служебный
    Eof,
}
```

Ключевые слова распознаются при лексировании идентификатора.

### `mod.rs` — структура `Lexer`

```rust
pub struct Lexer {
    chars: Vec<char>,
    pos:   usize,
}

impl Lexer {
    pub fn tokenize(&mut self) -> Vec<Token> { ... }
    fn next_token(&mut self) -> Token { ... }
}
```

**Особенности:**
- Числа с плавающей точкой: `3.14` → `Token::Float(3.14)`.
  Диапазонный оператор `1..5` не путается с `1.5`: лексер проверяет, что после `.` следует цифра и третий символ не `.`.
- Строки в двойных кавычках: `"текст"` → `Token::Str("текст")`.
- Комментарии `//` пропускаются до конца строки.
- Двусимвольные токены (`..=`, `..`, `+=`, `==` и т.д.) читаются с заглядыванием вперёд.

---

## Парсер (`src/parser/`)

### `ast.rs` — узлы AST

**Выражения (`Expr`):**
```
Number(i64) | Float(f64) | Str(String) | Ident(String)
Binary(Box<Expr>, BinOp, Box<Expr>)
Unary(UnaryOp, Box<Expr>)
Call { name, args }
MethodCall { obj, method, args }
FieldAccess { obj, field }
StructLit { name, fields }
ConstructorCall { class, args }
Input | InputFloat
```

**Операторы (`Stmt`):**
```
FnDecl { name, params, body }
StructDecl { name, fields }
ImplDecl { struct_name, methods }
ClassDecl { name, fields, methods }
Let { name, expr }             ← var name = expr
Assign { name, expr }
FieldAssign { obj, field, val }
Return(Expr) | Print(Expr)
If { cond, then, els }
While { cond, body }
DoWhile { body, cond }
For { var, from, to, inclusive, body }
Loop { body }
Break | Continue
Match { expr, arms }
Block(Vec<Stmt>) | Expr(Expr)
```

### `mod.rs` — рекурсивный нисходящий парсер

Иерархия приоритетов (от низкого к высокому):

```
parse_or        ||
parse_and       &&
parse_cmp       == != < <= > >=
parse_add       + -
parse_mul       * / %
parse_unary     - !
parse_postfix   expr.field  expr.method(args)
parse_call_base name(args)  StructName{...}  new Name(...)
parse_primary   литерал | ident | self | (expr)
```

**Важная деталь — разбор структурного литерала:**

`Ident { field: expr }` отличается от `match expr { ... }` с помощью функции `looks_like_struct_lit()`: структурный литерал проверяется только если после `{` идёт пара `ident :` или сразу `}` (пустая структура).

**Синтаксический сахар:**
- `for i in 0..3, j in 0..5 { }` → вложенные `Stmt::For` (деcахаризация при парсинге)
- `i += 1` → `Stmt::Assign { name: "i", expr: Binary(Ident("i"), Add, Number(1)) }`
- `init(params) { }` → `MethodDecl { name: "new", has_self: true, ... }`

---

## Кодогенерация (`src/codegen/`)

Использует библиотеку `inkwell` (безопасная обёртка над LLVM C API).

### `mod.rs` — структура `CodeGen`

```rust
pub struct CodeGen<'ctx> {
    ctx:           &'ctx Context,
    builder:       Builder<'ctx>,
    module:        Module<'ctx>,
    vars:          HashMap<String, Var<'ctx>>,   // текущие переменные
    i64_ty:        IntType<'ctx>,
    f64_ty:        FloatType<'ctx>,
    struct_types:  HashMap<String, StructType<'ctx>>,
    struct_fields: HashMap<String, Vec<(String, bool)>>,
    loop_stack:    Vec<(BasicBlock<'ctx>, BasicBlock<'ctx>)>,
}
```

### Трёхпроходная генерация (`generate_program`)

1. **Проход 0** — сбор типов структур и классов → `declare_struct()`
2. **Проход 1** — предварительное объявление функций и методов → позволяет взаимной рекурсии
3. **Проход 2** — генерация тел функций и методов

### Представление типов

| Тип языка  | LLVM тип   |
|------------|------------|
| `int`      | `i64`      |
| `float`    | `f64`      |
| `struct S` | `%S = type { ... }` (поля i64/f64) |

Структура всегда передаётся через указатель (stack-alloca + pointer).

### Методы: именование и `self`

Метод `method` структуры `Foo` становится LLVM-функцией `Foo_method`.

Если метод имеет `self`, первым параметром идёт `ptr` (указатель на структуру).

```
Orbitron:  pub func tick(self) { self.val += 1; }
LLVM:      define i64 @Counter_tick(ptr %0) { ... }
```

### Встроенные функции

В начале работы `CodeGen::new()` объявляет в модуле:

```c
int printf(char*, ...);
int scanf(char*, ...);
```

- `println(int v)` → `printf("%lld\n", v)`
- `println(float v)` → `printf("%g\n", v)`
- `println("str")` → `printf("%s\n", global_string)`
- `readInt()` → `scanf("%lld", &alloca); load alloca`
- `readFloat()` → `scanf("%lf", &alloca); load alloca`

### Типизация значений во время кодогенерации

Внутри `gen_expr()` все вычисленные значения оборачиваются в перечисление `Val`:

```rust
enum Val<'ctx> {
    Int(IntValue<'ctx>),
    Float(FloatValue<'ctx>),
    Struct(PointerValue<'ctx>, String), // ptr + имя типа
}
```

При смешанной арифметике `int + float` → оба приводятся к `f64`.

### Доступ к полям структуры

```
self.x = self.x + 1;
→
%gep = getelementptr %Vec2, ptr %self, i32 0, i32 <index>
%val = load i64, ptr %gep
%new = add i64 %val, 1
store i64 %new, ptr %gep
```

### Генерация циклов

```
for i in from..to:                   for i in from..=to:
┌─────────────────────────┐         ┌─────────────────────────┐
│ alloca i                │         │ alloca i                │
│ store from → i          │         │ store from → i          │
│ br for.cond             │         │ br for.cond             │
└─────────┬───────────────┘         └─────────┬───────────────┘
          ▼ for.cond                          ▼ for.cond
   i = load i                          i = load i
   cmp i64 SLT i, to                   cmp i64 SLE i, to
   br true → for.body, false → exit    br true → for.body, false → exit
          ▼ for.body                          ▼ for.body
       [тело]                             [тело]
       i += 1                             i += 1
       br for.cond                        br for.cond
          ▼ exit                                ▼ exit
```

`do..while`:
```
br do.body
do.body:
  [тело]
  br do.cond
do.cond:
  cmp ...
  br true → do.body, false → exit
exit:
```

---

## Пайплайн компиляции

```rust
// src/main.rs
fn run(config: Config) -> Result<(), CompileError> {
    let source = fs::read_to_string(&config.input)?;     // читаем .ot

    let tokens  = Lexer::tokenize(&source)?;             // лексер
    let program = Parser::new(tokens).parse_program()?;  // парсер → AST

    let ctx = Context::create();
    let mut cg = CodeGen::new("orbitron", &ctx);
    cg.generate_program(&program);                       // AST → LLVM IR
    cg.save_and_compile(&config.output, &opts)?;         // IR → бинарник

    Ok(())
}
```

`save_and_compile(output, opts)`:
1. `module.print_to_file("<output>.ll")` — запись LLVM IR
2. Если `--emit-llvm` → остановиться здесь
3. `llc <output>.ll -o <output>.s -relocation-model=pic` — компиляция в ассемблер
4. `clang <output>.s -o <output> -lm` — линковка
5. Если не `--save-temps` → удалить `<output>.ll` и `<output>.s`

## Интерфейс командной строки

```
ИСПОЛЬЗОВАНИЕ:
  orbitron [опции] <файл.ot>

ОПЦИИ:
  -h, --help         Вывести справку и выйти
      --version      Вывести версию и выйти
  -o <файл>          Имя выходного бинарника
                     (по умолчанию: имя исходника без расширения .ot)
      --emit-llvm    Сохранить LLVM IR в <output>.ll и не компилировать дальше
      --save-temps   Сохранить промежуточные файлы (<output>.ll, <output>.s)
  -v, --verbose      Выводить шаги компиляции
```

Примеры:
```bash
orbitron hello.ot                      # → ./hello
orbitron -o myapp hello.ot             # → ./myapp
orbitron --emit-llvm hello.ot          # → hello.ll (LLVM IR, без линковки)
orbitron --save-temps -o out hello.ot  # → out, сохранить out.ll и out.s
orbitron -v examples/fibonacci.ot      # подробный вывод шагов
```

---

## Зависимости (Cargo.toml)

| Зависимость           | Назначение                     |
|-----------------------|--------------------------------|
| `inkwell`             | Генерация LLVM IR (safe bindings) |
| `llvm-sys` (transitive)| C bindings для LLVM           |

Версия LLVM: 18.x (настраивается через feature flag inkwell).

---

## Расширение компилятора

### Добавить новый оператор
1. Добавить вариант в `Token` (`token.rs`) и распознать в лексере (`mod.rs`)
2. Добавить вариант в `Stmt` или `Expr` (`ast.rs`)
3. Распознать в парсере (`parser/mod.rs`)
4. Добавить кодогенерацию (`codegen/stmt.rs` или `codegen/expr.rs`)

### Добавить новый тип данных
1. Добавить вариант в `VarKind` и `Val` (`codegen/mod.rs`)
2. Расширить `declare_struct` и логику `gen_expr` / `gen_stmt`
3. Добавить соответствующий `BasicTypeEnum` в кодогенератор

### Добавить проверку типов
Между парсером и кодогенерацией можно вставить проход TypeChecker, который обходит AST и заполняет таблицу типов (`HashMap<String, Type>`), затем передаёт аннотированное AST в CodeGen.
