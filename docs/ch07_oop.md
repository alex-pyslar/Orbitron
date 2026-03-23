# Глава 7 — Объектно-ориентированное программирование

Orbitron поддерживает несколько стилей ООП, вдохновлённых разными традициями.
Вы можете использовать любой стиль — или несколько в одной программе.

---

## 7.1 — Обзор стилей ООП

| Стиль | Источник вдохновения | Данные + методы | Создание |
|-------|---------------------|-----------------|---------|
| `struct + impl` | Go, Rust | Раздельно | Структурный литерал (без `new`) |
| `class + init` | Java, C#, Kotlin | Вместе | `new ИмяКласса(аргументы)` |
| `class extends` | Java, Python | Наследование | `new Дочерний(аргументы)` |
| `trait` | Rust, Swift | Интерфейс | `impl Trait for Type` |

Оба основных стиля поддерживают:
- Именованные поля (данные)
- Методы (функции, работающие с данными)
- Статические методы (`static func`)
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
    public fn dist_sq(self): int {
        return self.x * self.x + self.y * self.y;
    }

    public fn move_by(self, dx: int, dy: int) {
        self.x = self.x + dx;
        self.y = self.y + dy;
    }

    public fn print_pos(self) {
        println($"Точка: x={self.x}, y={self.y}");
    }
}
```

### Параметр `self`

Каждый метод получает `self` как **первый параметр**. Это явное, а не неявное
`this`. Внутри метода обращайтесь к полям через `self.поле`:

```orbitron
public fn area(self): int {
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
    public fn area(self): int {
        return self.width * self.height;
    }

    public fn perimeter(self): int {
        return 2 * (self.width + self.height);
    }

    public fn is_square(self): int {
        return self.width == self.height ? 1 : 0;
    }

    public fn scale(self, factor: int) {
        self.width  = self.width  * factor;
        self.height = self.height * factor;
    }

    public fn describe(self) {
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
    public fn len_sq(self): int { return self.x**2 + self.y**2; }
    public fn dot(self, ox: int, oy: int): int {
        return self.x * ox + self.y * oy;
    }
}

impl Vec3 {
    public fn len_sq(self): int {
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

    public fn tick(self) {
        self.val = self.val + self.step;
    }

    public fn get(self): int {
        return self.val;
    }

    public fn reset(self) {
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

    public fn deposit(self, amount: int) {
        if (amount > 0) {
            self.balance = self.balance + amount;
        }
    }

    public fn withdraw(self, amount: int): int {
        if (amount <= 0)           { return -1; }  // неверная сумма
        if (self.balance < amount) { return -2; }  // недостаточно средств
        self.balance = self.balance - amount;
        return 0;  // успех
    }

    public fn get_balance(self): int {
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

## 7.4 — Статические методы (`static func`)

Статические методы принадлежат **типу**, а не конкретному экземпляру.
Они не принимают `self` и вызываются через синтаксис `Тип::метод(аргументы)`.

```orbitron
struct Vector {
    x: int,
    y: int,
}

impl Vector {
    // Обычный метод — принимает self
    public fn len_sq(self): int {
        return self.x * self.x + self.y * self.y;
    }

    // Статический метод — не принимает self
    static func zero() -> int {
        return 0;
    }

    static func from_angle(deg: int): int {
        // упрощение: возвращает код направления
        return deg / 90;
    }
}

func main() {
    var v = Vector { x: 3, y: 4 };
    println(v.len_sq());              // 25 — вызов обычного метода

    var z = Vector::zero();           // 0  — статический вызов
    var dir = Vector::from_angle(90); // 1  — статический вызов
    println(z);
    println(dir);
}
```

### Статические методы в классах

```orbitron
class Config {
    private max_users: int,
    private timeout:   int,

    init(mu: int, t: int) {
        self.max_users = mu;
        self.timeout   = t;
    }

    public fn get_max(self): int { return self.max_users; }
    public fn get_timeout(self): int { return self.timeout; }

    // Фабричный статический метод — создаёт «дефолтный» экземпляр
    static func default_timeout(): int {
        return 30;
    }

    static func default_max(): int {
        return 100;
    }
}

func main() {
    var t = Config::default_timeout();   // 30
    var m = Config::default_max();       // 100
    var cfg = new Config(m, t);

    println(cfg.get_max());      // 100
    println(cfg.get_timeout());  // 30
}
```

---

## 7.5 — Трейты (trait)

Трейт — это объявление интерфейса: набор сигнатур методов, которые обязан
реализовать любой тип, объявляющий поддержку трейта. Вдохновлён Rust и Swift:

```orbitron
trait Printable {
    func print_info(self);
}

trait Measurable {
    func area(self): int;
    func perimeter(self): int;
}
```

### impl Trait for Type

Чтобы реализовать трейт для конкретного типа, используйте синтаксис
`impl ИмяТрейта for ИмяТипа`:

```orbitron
struct Circle {
    radius: int,
}

impl Measurable for Circle {
    func area(self): int {
        // приближение: PI ≈ 314/100
        return 314 * self.radius * self.radius / 100;
    }

    func perimeter(self): int {
        return 628 * self.radius / 100;
    }
}

struct Square {
    side: int,
}

impl Measurable for Square {
    func area(self): int {
        return self.side * self.side;
    }

    func perimeter(self): int {
        return 4 * self.side;
    }
}

func main() {
    var c = Circle { radius: 5 };
    println(c.area());       // 78 (≈ π·25)
    println(c.perimeter());  // 31 (≈ 2π·5)

    var s = Square { side: 4 };
    println(s.area());       // 16
    println(s.perimeter());  // 16
}
```

### Трейт с несколькими методами

```orbitron
trait Comparable {
    func less_than(self, other: int): int;
    func equal_to(self, other: int): int;
}

struct Score {
    value: int,
}

impl Comparable for Score {
    func less_than(self, other: int): int {
        return self.value < other ? 1 : 0;
    }

    func equal_to(self, other: int): int {
        return self.value == other ? 1 : 0;
    }
}

func main() {
    var s1 = Score { value: 75 };
    var s2 = Score { value: 90 };

    println(s1.less_than(s2.value));   // 1 (75 < 90)
    println(s1.equal_to(75));          // 1
    println(s2.less_than(s1.value));   // 0
}
```

---

## 7.6 — Реализация операторов (`impl Add for Type`)

Orbitron позволяет переопределить операторы для своих типов, реализовав
специальный трейт-оператор. Используется тот же синтаксис `impl Trait for Type`:

```orbitron
struct Vec2 {
    x: int,
    y: int,
}

impl Add for Vec2 {
    func add(self, other_x: int, other_y: int): int {
        // Возвращает суммарный вектор — здесь выводим результат
        println($"Vec2({self.x + other_x}, {self.y + other_y})");
        return 0;
    }
}

impl Sub for Vec2 {
    func sub(self, other_x: int, other_y: int): int {
        println($"Vec2({self.x - other_x}, {self.y - other_y})");
        return 0;
    }
}

func main() {
    var a = Vec2 { x: 3, y: 4 };
    var b = Vec2 { x: 1, y: 2 };

    a.add(b.x, b.y);   // Vec2(4, 6)
    a.sub(b.x, b.y);   // Vec2(2, 2)
}
```

### Доступные трейты-операторы

| Трейт | Метод | Оператор |
|-------|-------|---------|
| `Add` | `func add(self, ...)` | `+` |
| `Sub` | `func sub(self, ...)` | `-` |
| `Mul` | `func mul(self, ...)` | `*` |
| `Div` | `func div(self, ...)` | `/` |
| `Neg` | `func neg(self)` | унарный `-` |
| `Eq`  | `func eq(self, ...)` | `==` |
| `Ord` | `func cmp(self, ...)` | `<`, `>`, `<=`, `>=` |

---

## 7.7 — Наследование классов (`class extends`)

Класс может наследовать от другого класса с помощью ключевого слова `extends`.
Дочерний класс получает все поля и методы родителя и может добавлять свои:

```orbitron
class Animal {
    private name: int,
    private age:  int,

    init(n: int, a: int) {
        self.name = n;
        self.age  = a;
    }

    public fn get_name(self): int { return self.name; }
    public fn get_age(self): int  { return self.age; }

    public fn speak(self) {
        println("...");
    }

    public fn describe(self) {
        var n = self.name;
        var a = self.age;
        println($"Животное #{n}, возраст {a}");
    }
}

class Dog extends Animal {
    private breed: int,

    init(n: int, a: int, b: int) {
        self.name  = n;
        self.age   = a;
        self.breed = b;
    }

    @override
    public fn speak(self) {
        println("Гав!");
    }

    public fn get_breed(self): int {
        return self.breed;
    }
}

class Cat extends Animal {
    private indoor: int,

    init(n: int, a: int, i: int) {
        self.name   = n;
        self.age    = a;
        self.indoor = i;
    }

    @override
    public fn speak(self) {
        println("Мяу!");
    }

    public fn is_indoor(self): int {
        return self.indoor;
    }
}

func main() {
    var dog = new Dog(1, 3, 42);   // name=1, age=3, breed=42
    var cat = new Cat(2, 5, 1);    // name=2, age=5, indoor=true

    dog.describe();     // Животное #1, возраст 3
    dog.speak();        // Гав!
    println(dog.get_breed());  // 42

    cat.describe();     // Животное #2, возраст 5
    cat.speak();        // Мяу!
    println(cat.is_indoor());  // 1
}
```

### Правила наследования

- Дочерний класс наследует все **публичные** методы родителя
- Поля родителя доступны через `self` (если объявлены в дочернем `init`)
- `@override` — рекомендуется для переопределяемых методов (аннотация)
- Поддерживается одиночное наследование (один родитель)
- Нельзя переопределить `init` — у каждого класса свой конструктор

### Многоуровневое наследование

```orbitron
class Shape {
    private color: int,

    init(c: int) {
        self.color = c;
    }

    public fn get_color(self): int { return self.color; }
}

class Polygon extends Shape {
    private sides: int,

    init(c: int, s: int) {
        self.color = c;
        self.sides = s;
    }

    public fn get_sides(self): int { return self.sides; }
}

class RegularPolygon extends Polygon {
    private side_len: int,

    init(c: int, s: int, l: int) {
        self.color    = c;
        self.sides    = s;
        self.side_len = l;
    }

    public fn perimeter(self): int {
        return self.sides * self.side_len;
    }
}

func main() {
    var hex = new RegularPolygon(3, 6, 10);   // color=3, sides=6, len=10
    println(hex.get_color());    // 3
    println(hex.get_sides());    // 6
    println(hex.perimeter());    // 60
}
```

---

## 7.8 — Модификаторы доступа

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

    public fn push(self, val: int) {
        if (self.top == 0) { self.data_0 = val; }
        if (self.top == 1) { self.data_1 = val; }
        if (self.top == 2) { self.data_2 = val; }
        if (self.top == 3) { self.data_3 = val; }
        self.top += 1;
    }

    public fn pop(self): int {
        self.top -= 1;
        if (self.top == 0) { return self.data_0; }
        if (self.top == 1) { return self.data_1; }
        if (self.top == 2) { return self.data_2; }
        return self.data_3;
    }

    public fn size(self): int {
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

## 7.9 — Когда использовать какой стиль

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

### Используйте `trait`, когда:
- Нужно описать интерфейс без реализации
- Несколько типов должны поддерживать одни и те же методы
- Реализуете обобщённые алгоритмы (паттерн «утиная типизация»)

### Используйте `extends`, когда:
- Один тип является специализацией другого («собака — это животное»)
- Хотите переиспользовать код родителя и добавить дополнительное поведение

---

## 7.10 — Таблица сравнения стилей ООП

| Аспект | `struct + impl` | `class + init` | `class extends` | `trait` |
|--------|----------------|----------------|-----------------|---------|
| Источник | Go, Rust | Java, C# | Java, Python | Rust, Swift |
| Объявление полей | `struct Name {}` | `class Name {}` | `class Dog extends Animal {}` | — |
| Методы | `impl Name {}` | Внутри `class {}` | Внутри `class {}` | Сигнатуры |
| Конструктор | Структурный литерал | `init(params)` | `init(params)` | — |
| Создание | `Name { field: val }` | `new Name(args)` | `new Dog(args)` | — |
| Реализация | — | — | — | `impl Trait for Type` |
| `self` | Явный параметр | Явный параметр | Явный параметр | Явный параметр |
| Статические | `static func f()` | `static func f()` | `static func f()` | — |

---

## 7.11 — Полный пример ООП: Геометрическая библиотека

```orbitron
// examples/05_oop/structs.ot

trait Shape {
    func area(self): int;
    func perimeter(self): int;
}

struct Circle {
    radius: int,
    cx:     int,
    cy:     int,
}

impl Shape for Circle {
    func area(self): int {
        return 314 * self.radius * self.radius / 100;
    }

    func perimeter(self): int {
        return 628 * self.radius / 100;
    }
}

impl Circle {
    public fn contains(self, x: int, y: int): int {
        var dx = x - self.cx;
        var dy = y - self.cy;
        return dx*dx + dy*dy <= self.radius * self.radius ? 1 : 0;
    }

    public fn describe(self) {
        var a = self.area();
        println($"Окружность r={self.radius} в ({self.cx},{self.cy}), площадь≈{a}");
    }
}

struct Triangle {
    a: int,
    b: int,
    c: int,
}

impl Shape for Triangle {
    func area(self): int {
        // Формула Герона: s*(s-a)*(s-b)*(s-c), приблизительно через периметр
        return self.a * self.b / 2;   // упрощение для прямоугольного треугольника
    }

    func perimeter(self): int {
        return self.a + self.b + self.c;
    }
}

impl Triangle {
    public fn is_valid(self): int {
        if (self.a + self.b <= self.c) { return 0; }
        if (self.b + self.c <= self.a) { return 0; }
        if (self.a + self.c <= self.b) { return 0; }
        return 1;
    }

    public fn is_equilateral(self): int {
        return (self.a == self.b && self.b == self.c) ? 1 : 0;
    }

    public fn is_isosceles(self): int {
        return (self.a == self.b || self.b == self.c || self.a == self.c) ? 1 : 0;
    }
}

func main() {
    var c = Circle { radius: 5, cx: 0, cy: 0 };
    c.describe();
    println(c.area());           // 78
    println(c.contains(3, 4));   // 1 (внутри: 3^2+4^2=25 <= 25)
    println(c.contains(4, 4));   // 0 (снаружи: 4^2+4^2=32 > 25)

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
