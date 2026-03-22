# Глава 10 — Проекты и модули

Когда программа растёт и перестаёт помещаться в один файл, наступает время создать
**проект** — директорию с файлом манифеста и несколькими исходными файлами.

---

## 10.1 — Два режима работы

### Режим одного файла

```bash
orbitron hello.ot         # компилирует один .ot файл → ./hello
```

Подходит для небольших скриптов и экспериментов.

### Режим проекта

```bash
orbitron new myapp        # создать новый проект
cd myapp
orbitron build            # скомпилировать весь проект
orbitron run              # собрать и запустить
```

Подходит для проектов с несколькими файлами.

---

## 10.2 — Создание проекта

```bash
orbitron new calculator
```

Создаётся следующая структура:

```
calculator/
├── orbitron.toml       ← манифест проекта
├── src/
│   └── main.ot         ← точка входа (содержит func main())
└── bin/                ← сюда попадает скомпилированный бинарник
```

Сгенерированный `src/main.ot`:

```orbitron
func main() {
    println("Hello from calculator!");
}
```

---

## 10.3 — Файл манифеста orbitron.toml

`orbitron.toml` — конфигурационный файл проекта:

```toml
[project]
name    = "calculator"
version = "0.1.0"

[build]
main    = "src/main.ot"       # точка входа
output  = "bin/calculator"    # путь к выходному файлу
backend = "llvm"              # "llvm" или "jvm"
```

### Секция `[project]`

| Поле | Тип | Описание |
|------|-----|----------|
| `name` | строка | Имя проекта |
| `version` | строка | Версия в формате semver |

### Секция `[build]`

| Поле | Тип | По умолчанию | Описание |
|------|-----|-------------|----------|
| `main` | строка | — | Путь к файлу с `func main()` |
| `output` | строка | `bin/<name>` | Путь к выходному файлу |
| `backend` | строка | `"llvm"` | Бэкенд: `"llvm"` или `"jvm"` |

---

## 10.4 — Структура директорий

Рекомендуемая структура для проектов среднего размера:

```
myproject/
├── orbitron.toml
├── src/
│   ├── main.ot         ← точка входа, func main()
│   ├── math.ot         ← математические утилиты
│   ├── io.ot           ← ввод/вывод
│   └── models.ot       ← типы данных
└── bin/                ← скомпилированный бинарник
```

Каждый файл `.ot` в `src/` является **модулем**, который можно импортировать
в другие файлы.

---

## 10.5 — Инструкция import

```orbitron
import "имя_модуля";
```

Загружает файл `src/имя_модуля.ot` и объединяет его AST с текущим файлом.

### Правила импорта

- `import "math"` → загружает `src/math.ot`
- `import "net/http"` → загружает `src/net/http.ot` (поддиректории)
- `import "std/math"` → загружает стандартную библиотеку `stdlib/math.ot`
- Импорты разрешаются **до** кодогенерации: компилятор обходит все файлы
  и объединяет их AST
- Дублированные импорты игнорируются (каждый файл импортируется не более одного раза)
- Циклические импорты приводят к ошибке компиляции

### Расположение import

Инструкции `import` обычно размещаются в начале файла:

```orbitron
import "std/math";
import "std/algo";
import "utils";
import "models";

func main() {
    // ...
}
```

---

## 10.6 — Многофайловый проект: Калькулятор

Создадим полноценный многофайловый проект.

### Структура

```
calculator/
├── orbitron.toml
└── src/
    ├── main.ot     ← точка входа
    └── math.ot     ← математический модуль
```

### orbitron.toml

```toml
[project]
name    = "calculator"
version = "1.0.0"

[build]
main   = "src/main.ot"
output = "bin/calculator"
```

### src/math.ot

```orbitron
// Математический модуль — базовые операции

func add(a: int, b: int): int { return a + b; }
func sub(a: int, b: int): int { return a - b; }
func mul(a: int, b: int): int { return a * b; }

func div(a: int, b: int): int {
    unless (b == 0) {
        return a / b;
    }
    return 0;   // деление на ноль → 0
}

func mod(a: int, b: int): int {
    unless (b == 0) {
        return a % b;
    }
    return 0;
}

func pow(base: int, exp: int): int {
    if (exp == 0) { return 1; }
    var result = 1;
    repeat exp { result = result * base; }
    return result;
}

func abs(n: int): int {
    return n >= 0 ? n : -n;
}
```

### src/main.ot

```orbitron
import "math";

func print_result(op: int, a: int, b: int, result: int) {
    match op {
        1 => { println($"  {a} + {b} = {result}"); }
        2 => { println($"  {a} - {b} = {result}"); }
        3 => { println($"  {a} * {b} = {result}"); }
        4 => { println($"  {a} / {b} = {result}"); }
        5 => { println($"  {a} % {b} = {result}"); }
        6 => { println($"  {a} ^ {b} = {result}"); }
        _ => { println("  неизвестная операция"); }
    }
}

func main() {
    println("=== Калькулятор Orbitron ===");

    var a = 10;
    var b = 3;

    print_result(1, a, b, add(a, b));   // 10 + 3 = 13
    print_result(2, a, b, sub(a, b));   // 10 - 3 = 7
    print_result(3, a, b, mul(a, b));   // 10 * 3 = 30
    print_result(4, a, b, div(a, b));   // 10 / 3 = 3
    print_result(5, a, b, mod(a, b));   // 10 % 3 = 1
    print_result(6, a, b, pow(a, b));   // 10 ^ 3 = 1000

    // Тест деления на ноль
    var c = div(5, 0);
    println($"  5 / 0 = {c}");   // 0 (защищено)

    // Абсолютное значение
    var neg = abs(-42);
    println($"  |{-42}| = {neg}");   // 42
}
```

### Сборка и запуск

```bash
cd calculator
orbitron build
./bin/calculator
```

---

## 10.7 — Многофайловый проект: Геометрия

Более сложный пример с несколькими модулями.

### Структура

```
geometry/
├── orbitron.toml
└── src/
    ├── main.ot
    ├── vectors.ot
    └── shapes.ot
```

### src/vectors.ot

```orbitron
// 2D-векторы

struct Vec2 {
    x: int,
    y: int,
}

impl Vec2 {
    pub func add(self, ox: int, oy: int): int {
        // Возвращает x-компоненту суммы (упрощённо)
        return self.x + ox;
    }

    pub func len_sq(self): int {
        return self.x * self.x + self.y * self.y;
    }

    pub func dot(self, ox: int, oy: int): int {
        return self.x * ox + self.y * oy;
    }

    pub func scale(self, factor: int) {
        self.x = self.x * factor;
        self.y = self.y * factor;
    }

    pub func describe(self) {
        println($"Vec2({self.x}, {self.y})");
    }
}
```

### src/shapes.ot

```orbitron
// Геометрические фигуры

import "vectors";

struct Circle {
    center_x: int,
    center_y: int,
    radius:   int,
}

impl Circle {
    pub func area_approx(self): int {
        return 314 * self.radius * self.radius / 100;
    }

    pub func perimeter_approx(self): int {
        return 628 * self.radius / 100;
    }

    pub func contains(self, px: int, py: int): int {
        var dx = px - self.center_x;
        var dy = py - self.center_y;
        return dx*dx + dy*dy <= self.radius * self.radius ? 1 : 0;
    }
}

struct Rect {
    x:      int,
    y:      int,
    width:  int,
    height: int,
}

impl Rect {
    pub func area(self):      int { return self.width * self.height; }
    pub func perimeter(self): int { return 2 * (self.width + self.height); }
    pub func is_square(self): int { return self.width == self.height ? 1 : 0; }

    pub func contains(self, px: int, py: int): int {
        var in_x = px >= self.x && px <= self.x + self.width;
        var in_y = py >= self.y && py <= self.y + self.height;
        return in_x && in_y ? 1 : 0;
    }
}
```

### src/main.ot

```orbitron
import "vectors";
import "shapes";

func main() {
    println("=== Геометрия ===");

    // Векторы
    var v = Vec2 { x: 3, y: 4 };
    v.describe();
    println(v.len_sq());     // 25 (3^2 + 4^2)
    println(v.dot(1, 0));    // 3

    v.scale(2);
    v.describe();            // Vec2(6, 8)

    // Окружность
    var c = Circle { center_x: 0, center_y: 0, radius: 10 };
    println(c.area_approx());       // ~314
    println(c.contains(6, 8));      // 1 (6^2+8^2=100 <= 100)
    println(c.contains(7, 8));      // 0 (7^2+8^2=113 > 100)

    // Прямоугольник
    var r = Rect { x: 0, y: 0, width: 5, height: 3 };
    println(r.area());              // 15
    println(r.perimeter());         // 16
    println(r.is_square());         // 0
    println(r.contains(3, 2));      // 1
    println(r.contains(6, 2));      // 0
}
```

---

## 10.8 — Разрешение импортов

При компиляции проекта компилятор:

1. **Читает** `orbitron.toml` → находит точку входа (`main`)
2. **Парсит** точку входа
3. **Находит** все инструкции `import` в AST
4. **Рекурсивно парсит** импортированные файлы
5. **Отслеживает** уже импортированные пути (через `HashSet<PathBuf>`)
6. **Объединяет** все AST в один перед кодогенерацией

```
main.ot
  import "math"    → src/math.ot
  import "models"  → src/models.ot
                     → import "math"  (игнорируется — уже загружен)

Результирующий AST: [math.ot] + [models.ot] + [main.ot]
```

### Правило: нет порядка объявления

Поскольку все файлы объединяются перед кодогенерацией, порядок импортов
и порядок объявления функций не имеют значения:

```orbitron
// src/main.ot

import "utils";   // utils.ot может использовать функции из main.ot
                  // и наоборот — без проблем

func main() {
    helper();   // объявлена в utils.ot
}
```

### Обнаружение циклических импортов

Циклические зависимости обнаруживаются и приводят к ошибке:

```
main.ot imports a.ot
a.ot    imports b.ot
b.ot    imports main.ot   ← ОШИБКА: циклический импорт
```

---

## 10.9 — Команды для проектов

```bash
# Создать новый проект
orbitron new myapp

# В директории проекта:
orbitron build               # компилировать → bin/myapp
orbitron run                 # компилировать + запустить
orbitron build -o bin/debug  # своё имя файла
orbitron build --backend jvm # скомпилировать в .jar
orbitron build --emit-llvm   # остановиться на LLVM IR
orbitron build -v            # подробный вывод каждого шага

# Из родительской директории:
orbitron build --project ./myapp    # (поиск orbitron.toml автоматический)
```

Команда `build` ищет `orbitron.toml` начиная с текущей директории и поднимаясь выше.

---

← [Глава 9 — Стандартная библиотека](ch09_stdlib.md) | [Глава 11 — Низкоуровневое программирование →](ch11_lowlevel.md)
