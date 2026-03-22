# Глава 7 — Объектно-ориентированное программирование

Orbitron поддерживает два различных стиля ООП, каждый вдохновлён разной традицией.
Вы можете использовать любой стиль — или оба в одной программе.

---

## 7.1 — Два подхода к ООП

| Стиль | Источник вдохновения | Данные + методы | Создание |
|-------|---------------------|-----------------|---------|
| `struct + impl` | Go, Rust | Раздельно | Структурный литерал (без `new`) |
| `class + init` | Java, C#, Kotlin | Вместе | `new ИмяКласса(аргументы)` |

Оба стиля поддерживают:
- Именованные поля (данные)
- Методы (функции, работающие с данными)
- Модификаторы доступа `pub` и `private`
- Явный параметр `self`

---

## 7.2 — struct + impl (стиль Go / Rust)

В этом стиле **данные** определяются в блоке `struct`, а **методы** — отдельно
в блоке `impl`.

### Определение структуры

```orbitron
struct Point {
    x: int,
    y: int,
}
```

Поля разделяются запятыми. У каждого поля есть имя и тип (`int` или `float`).

### Реализация методов

```orbitron
impl Point {
    pub func dist_sq(self): int {
        return self.x * self.x + self.y * self.y;
    }

    pub func move_by(self, dx: int, dy: int) {
        self.x = self.x + dx;
        self.y = self.y + dy;
    }

    pub func print_pos(self) {
        println($"Точка: x={self.x}, y={self.y}");
    }
}
```

### Параметр `self`

Каждый метод получает `self` как **первый параметр**. Это явное, а не неявное
`this`. Внутри метода обращайтесь к полям через `self.поле`:

```orbitron
pub func area(self): int {
    return self.width * self.height;
}
```

### Создание экземпляров структуры

Структуры создаются **структурным литералом** — без ключевого слова `new`:

```orbitron
var p = Point { x: 3, y: 4 };
```

### Вызов методов

```orbitron
var p = Point { x: 3, y: 4 };

println(p.dist_sq());   // 25
p.move_by(1, -1);
p.print_pos();          // Точка: x=4, y=3
```

### Полный пример

```orbitron
struct Rectangle {
    width:  int,
    height: int,
}

impl Rectangle {
    pub func area(self): int {
        return self.width * self.height;
    }

    pub func perimeter(self): int {
        return 2 * (self.width + self.height);
    }

    pub func is_square(self): int {
        return self.width == self.height ? 1 : 0;
    }

    pub func scale(self, factor: int) {
        self.width  = self.width  * factor;
        self.height = self.height * factor;
    }

    pub func describe(self) {
        var a = self.area();
        var p = self.perimeter();
        println($"Прямоугольник {self.width}x{self.height}: площадь={a}, периметр={p}");
    }
}

func main() {
    var r = Rectangle { width: 5, height: 3 };
    r.describe();           // Прямоугольник 5x3: площадь=15, периметр=16
    println(r.is_square()); // 0

    r.scale(2);
    r.describe();           // Прямоугольник 10x6: площадь=60, периметр=32

    var sq = Rectangle { width: 7, height: 7 };
    println(sq.is_square()); // 1
}
```

### Несколько структур

Можно определить сколько угодно структур:

```orbitron
struct Vec2 { x: int, y: int }
struct Vec3 { x: int, y: int, z: int }

impl Vec2 {
    pub func len_sq(self): int { return self.x**2 + self.y**2; }
    pub func dot(self, ox: int, oy: int): int {
        return self.x * ox + self.y * oy;
    }
}

impl Vec3 {
    pub func len_sq(self): int {
        return self.x**2 + self.y**2 + self.z**2;
    }
}

func main() {
    var v2 = Vec2 { x: 3, y: 4 };
    println(v2.len_sq());          // 25
    println(v2.dot(1, 0));         // 3

    var v3 = Vec3 { x: 1, y: 2, z: 2 };
    println(v3.len_sq());          // 9
}
```

---

## 7.3 — class + init (стиль Java / C#)

В этом стиле поля и методы определяются **внутри одного блока `class`**.
Специальный метод `init` играет роль конструктора.

### Определение класса

```orbitron
class Counter {
    private val:  int,
    private step: int,

    init(start: int, s: int) {
        self.val  = start;
        self.step = s;
    }

    pub func tick(self) {
        self.val = self.val + self.step;
    }

    pub func get(self): int {
        return self.val;
    }

    pub func reset(self) {
        self.val = 0;
    }
}
```

### Конструктор `init`

Метод `init` вызывается автоматически при создании экземпляра через `new`.
Он инициализирует поля объекта:

```orbitron
init(x: int, y: int) {
    self.x = x;
    self.y = y;
}
```

- У `init` нет возвращаемого типа
- Внутри `init` доступен `self` для установки полей
- Конструктор вызывается через `new ИмяКласса(аргументы)`

### Создание экземпляров класса

Используйте ключевое слово `new`:

```orbitron
var c = new Counter(0, 5);   // start=0, step=5
```

### Вызов методов

```orbitron
var c = new Counter(0, 5);
c.tick();
c.tick();
println(c.get());   // 10

c.reset();
println(c.get());   // 0
```

### Полный пример — Банковский счёт

```orbitron
class BankAccount {
    private balance: int,

    init(initial_balance: int) {
        self.balance = initial_balance;
    }

    pub func deposit(self, amount: int) {
        if (amount > 0) {
            self.balance = self.balance + amount;
        }
    }

    pub func withdraw(self, amount: int): int {
        if (amount <= 0)           { return -1; }  // неверная сумма
        if (self.balance < amount) { return -2; }  // недостаточно средств
        self.balance = self.balance - amount;
        return 0;  // успех
    }

    pub func get_balance(self): int {
        return self.balance;
    }
}

func main() {
    var acc = new BankAccount(1000);
    println(acc.get_balance());   // 1000

    acc.deposit(500);
    println(acc.get_balance());   // 1500

    var r1 = acc.withdraw(200);
    println(r1);                  // 0 (успех)
    println(acc.get_balance());   // 1300

    var r2 = acc.withdraw(2000);
    println(r2);                  // -2 (недостаточно средств)
    println(acc.get_balance());   // 1300 (без изменений)
}
```

---

## 7.4 — Модификаторы доступа

Оба стиля (struct+impl и class) поддерживают модификаторы доступа:

| Ключевое слово | Значение | Примечание |
|----------------|----------|-----------|
| `pub` | Публичный | Доступен снаружи |
| `private` | Приватный | Концептуально внутренний |

> **Важно:** Модификаторы доступа фиксируются в AST, но пока не проверяются
> компилятором. Они служат документацией и обозначением намерений.

```orbitron
class Stack {
    private data_0: int,
    private data_1: int,
    private data_2: int,
    private data_3: int,
    private top: int,

    init() {
        self.top = 0;
    }

    pub func push(self, val: int) {
        if (self.top == 0) { self.data_0 = val; }
        if (self.top == 1) { self.data_1 = val; }
        if (self.top == 2) { self.data_2 = val; }
        if (self.top == 3) { self.data_3 = val; }
        self.top += 1;
    }

    pub func pop(self): int {
        self.top -= 1;
        if (self.top == 0) { return self.data_0; }
        if (self.top == 1) { return self.data_1; }
        if (self.top == 2) { return self.data_2; }
        return self.data_3;
    }

    pub func size(self): int {
        return self.top;
    }
}

func main() {
    var s = new Stack();
    s.push(10);
    s.push(20);
    s.push(30);
    println(s.size());  // 3
    println(s.pop());   // 30
    println(s.pop());   // 20
    println(s.size());  // 1
}
```

---

## 7.5 — Когда использовать какой стиль

### Используйте `struct + impl`, когда:
- Ваши данные — это чистый тип значений (точка, вектор, размер, цвет)
- Вы хотите стиль композиции в духе Go/Rust
- Логика инициализации не нужна (структурного литерала достаточно)
- Вы предпочитаете чёткое разделение данных и методов

### Используйте `class + init`, когда:
- Нужен конструктор с логикой инициализации
- Вы хотите инкапсуляцию в стиле Java/Kotlin
- Объект имеет инварианты, которые нужно установить при создании
- Вы пришли из Java/C# и предпочитаете такой стиль

---

## 7.6 — Таблица сравнения стилей ООП

| Аспект | `struct + impl` | `class + init` |
|--------|----------------|----------------|
| Источник вдохновения | Go, Rust | Java, C#, Kotlin |
| Объявление полей | `struct Name { field: type }` | `class Name { field: type, }` |
| Объявление методов | `impl Name { pub func f(self) {} }` | Внутри `class Name { pub func f(self) {} }` |
| Конструктор | Не нужен | `init(params) { self.field = ...; }` |
| Создание экземпляра | `Name { field: val }` | `new Name(args)` |
| `self` | Явный параметр | Явный параметр |
| Модификаторы доступа | `pub` / `private` | `pub` / `private` |

---

## 7.7 — Полный пример ООП: Геометрическая библиотека

```orbitron
// examples/05_oop/structs.ot

struct Circle {
    radius: int,
    cx:     int,
    cy:     int,
}

impl Circle {
    // Площадь * 100 (чтобы избежать float) ≈ 314 * r^2 / 100
    pub func area_x100(self): int {
        return 314 * self.radius * self.radius / 100;
    }

    pub func contains(self, x: int, y: int): int {
        var dx = x - self.cx;
        var dy = y - self.cy;
        return dx*dx + dy*dy <= self.radius * self.radius ? 1 : 0;
    }

    pub func describe(self) {
        var a = self.area_x100();
        println($"Окружность r={self.radius} в ({self.cx},{self.cy}), приближ. площадь={a}/100");
    }
}

struct Triangle {
    a: int,
    b: int,
    c: int,
}

impl Triangle {
    pub func is_valid(self): int {
        if (self.a + self.b <= self.c) { return 0; }
        if (self.b + self.c <= self.a) { return 0; }
        if (self.a + self.c <= self.b) { return 0; }
        return 1;
    }

    pub func perimeter(self): int {
        return self.a + self.b + self.c;
    }

    pub func is_equilateral(self): int {
        return (self.a == self.b && self.b == self.c) ? 1 : 0;
    }

    pub func is_isosceles(self): int {
        return (self.a == self.b || self.b == self.c || self.a == self.c) ? 1 : 0;
    }
}

func main() {
    var c = Circle { radius: 5, cx: 0, cy: 0 };
    c.describe();
    println(c.contains(3, 4));    // 1 (внутри: 3^2+4^2=25 <= 25)
    println(c.contains(4, 4));    // 0 (снаружи: 4^2+4^2=32 > 25)

    var t1 = Triangle { a: 3, b: 4, c: 5 };
    println(t1.is_valid());       // 1
    println(t1.perimeter());      // 12
    println(t1.is_equilateral()); // 0

    var t2 = Triangle { a: 6, b: 6, c: 6 };
    println(t2.is_equilateral()); // 1
    println(t2.is_isosceles());   // 1
}
```

---

← [Глава 6 — Коллекции](ch06_collections.md) | [Глава 8 — Специальные возможности →](ch08_features.md)
