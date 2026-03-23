# Глава 5 — Функции

Функции — фундаментальные строительные блоки любой программы на Orbitron.
Они позволяют именовать, переиспользовать и компоновать логику.

---

## 5.1 — Объявление функций

Функция объявляется с ключевым словом `fn`:

```orbitron
fn имя(параметр1: тип, параметр2: тип): тип_возврата {
    // тело
    return значение;
}
```

Минимальный пример:

```orbitron
fn greet() {
    println!("Привет из функции!");
}
```

Функция с параметрами и возвращаемым значением:

```orbitron
fn add(a: i64, b: i64): i64 {
    return a + b;
}
```

### Fat-arrow тела (однострочные функции)

Для простых функций можно использовать синтаксис `=>`:

```orbitron
fn double(x: i64): i64 => x * 2;
fn square(x: i64): i64 => x * x;
fn max2(a: i64, b: i64): i64 => a > b ? a : b;
```

### Вызов функций

```orbitron
fn main() {
    greet();               // вызов без аргументов
    var sum = add(3, 4);   // вызов с аргументами, захват возвращаемого значения
    println!(sum);         // 7
}
```

---

## 5.2 — Параметры и возвращаемые значения

Аннотации типов на параметрах и возвращаемом значении **необязательны**. Компилятор
пока не проверяет их строго — они служат документацией.

```orbitron
// Полностью аннотировано (рекомендуется для документации)
fn multiply(a: i64, b: i64): i64 {
    return a * b;
}

// Fat-arrow для однострочных функций
fn multiply_short(a: i64, b: i64): i64 => a * b;

// Без аннотаций (короче, тоже работает)
fn multiply2(a, b) {
    return a * b;
}
```

### Аннотация возвращаемого типа через `->`

Помимо синтаксиса `: тип` после скобок, можно использовать стрелку `->`:

```orbitron
fn square(n: i64) -> i64 {
    return n * n;
}

fn greet_user(name: i64) -> i64 {
    println!("Привет, пользователь \{name}!");
    return 0;
}
```

Оба стиля (`): i64` и `) -> i64`) равнозначны.

### Возвращаемое значение

Используйте `return`, чтобы выйти из функции и вернуть значение:

```orbitron
fn max_of(a: i64, b: i64): i64 {
    if (a > b) { return a; }
    return b;
}
```

Функция без инструкции `return` неявно возвращает `0`.

### Множественные return

Функция может иметь несколько инструкций `return`:

```orbitron
fn classify(n: i64): i64 {
    if (n > 0) { return 1; }   // положительное
    if (n < 0) { return -1; }  // отрицательное
    return 0;                  // ноль
}
```

---

## 5.3 — Параметры по умолчанию

Параметрам функции можно задать **значение по умолчанию**. Если аргумент
не передан при вызове, используется значение по умолчанию:

```orbitron
fn greet(times: i64, gap: i64 = 1) {
    for i in 0..times {
        println!("Привет!");
        // gap используется как задержка (здесь для примера)
    }
}

fn main() {
    greet(3);      // gap = 1 (по умолчанию)
    greet(3, 2);   // gap = 2 (явно задан)
}
```

### Несколько параметров по умолчанию

```orbitron
fn create_rect(width: i64, height: i64 = 10, filled: i64 = 0): i64 {
    var area = width * height;
    println!("прямоугольник \{width}x\{height}, заполнен=\{filled}, площадь=\{area}");
    return area;
}

fn main() {
    create_rect(5);           // width=5, height=10, filled=0
    create_rect(5, 8);        // width=5, height=8,  filled=0
    create_rect(5, 8, 1);     // width=5, height=8,  filled=1
}
```

### Правила параметров по умолчанию

- Параметры со значением по умолчанию должны стоять **после** обязательных параметров
- Значение по умолчанию должно быть числовым литералом
- При вызове параметры с умолчанием можно опустить справа налево

```orbitron
// ОК — параметры со значением по умолчанию идут в конце
fn connect(host: i64, port: i64 = 80, timeout: i64 = 30) { }

// ОШИБКА — обязательный параметр после параметра с умолчанием
// fn broken(x: i64 = 0, y: i64) { }
```

---

## 5.4 — Функция main

Каждая программа должна содержать функцию `main` — это точка входа:

```orbitron
fn main() {
    // программа начинается здесь
    println!("Привет");
}
```

`main` не принимает параметров и ничего не возвращает (неявно возвращает 0 в ОС).

---

## 5.5 — Область видимости

### Локальные переменные

Переменные, объявленные внутри функции, **локальны** для этой функции:

```orbitron
fn compute() {
    var x = 10;   // локальна для compute
    println!(x);
}

fn main() {
    compute();
    // println!(x);   // ОШИБКА — x здесь не видна
}
```

### Глобальные константы

Константы, объявленные на верхнем уровне (вне всех функций), видны всем функциям:

```orbitron
#const LIMIT: i64 = 100;

fn check(n: i64): i64 => n < LIMIT ? 1 : 0;

fn main() {
    println!(check(50));    // 1
    println!(check(200));   // 0
    println!(LIMIT);        // 100
}
```

### Функции, вызывающие друг друга

Функции могут вызывать другие функции, объявленные в любом месте файла.
Порядок объявления не важен — компилятор делает проход для предварительных объявлений:

```orbitron
fn main() {
    println!(helper());   // ОК — helper объявлена ниже
}

fn helper(): i64 => 42;
```

---

## 5.6 — Рекурсия

Функция может вызывать саму себя. Orbitron полностью поддерживает рекурсию:

```orbitron
fn factorial(n: i64): i64 {
    if (n <= 1) { return 1; }
    return n * factorial(n - 1);
}

fn main() {
    println!(factorial(1));   // 1
    println!(factorial(5));   // 120
    println!(factorial(10));  // 3628800
}
```

### Числа Фибоначчи (рекурсивно)

```orbitron
fn fib(n: i64): i64 {
    if (n <= 1) { return n; }
    return fib(n - 1) + fib(n - 2);
}
```

### Числа Фибоначчи (итеративно — эффективнее)

Рекурсия может быть медленной для больших входных данных. Используйте
итерацию, когда важна производительность:

```orbitron
fn fib_iter(n: i64): i64 {
    if (n <= 1) { return n; }
    mut a = 0;
    mut b = 1;
    mut i = 2;
    while (i <= n) {
        var tmp = a + b;
        a = b;
        b = tmp;
        i += 1;
    }
    return b;
}
```

### НОД (алгоритм Евклида)

```orbitron
fn gcd(a: i64, b: i64): i64 {
    if (b == 0) { return a; }
    return gcd(b, a % b);
}

fn main() {
    println!(gcd(48, 18));    // 6
    println!(gcd(100, 75));   // 25
}
```

---

## 5.7 — Лямбда-выражения (замыкания)

Лямбда — это анонимная функция, объявленная прямо в месте использования.
Используется синтаксис `|параметры| тело`:

```orbitron
var double = |x| x * 2;
var add    = |a, b| a + b;
var square = |x| x * x;
```

### Вызов лямбды

```orbitron
var double = |x| x * 2;
println!(double(5));    // 10
println!(double(21));   // 42
```

### Лямбды с несколькими параметрами

```orbitron
var clamp = |x, lo, hi| x < lo ? lo : x > hi ? hi : x;

println!(clamp(5, 0, 10));    // 5
println!(clamp(-3, 0, 10));   // 0
println!(clamp(15, 0, 10));   // 10
```

### Лямбды в конвейерах

Лямбды удобно комбинировать с оператором `|>`:

```orbitron
var double  = |x| x * 2;
var inc     = |x| x + 1;
var negate  = |x| -x;

var result = 5 |> double |> inc |> negate;
println!(result);   // -(5*2+1) = -11
```

### Многострочная лямбда

Если тело лямбды сложнее одного выражения, используйте блок `{ ... }`:

```orbitron
var process = |x| {
    var doubled = x * 2;
    var shifted = doubled + 10;
    return shifted;
};

println!(process(5));    // 20
println!(process(15));   // 40
```

---

## 5.8 — Статические методы и синтаксис `::`

Статические методы принадлежат типу, а не конкретному экземпляру. Они
не принимают `self` и вызываются через синтаксис `Тип::метод(аргументы)`:

```orbitron
struct MathUtils { }

impl MathUtils {
    pub static fn square(x: i64): i64 => x * x;
    pub static fn max(a: i64, b: i64): i64 => a > b ? a : b;
    pub static fn clamp(x: i64, lo: i64, hi: i64): i64 => x < lo ? lo : x > hi ? hi : x;
}

fn main() {
    println!(MathUtils::square(7));          // 49
    println!(MathUtils::max(3, 9));          // 9
    println!(MathUtils::clamp(150, 0, 100)); // 100
}
```

### Статические методы в классах

Статические методы можно объявлять и в классах через `class + init`:

```orbitron
class Counter {
    private val: i64,

    init(start: i64) {
        self.val = start;
    }

    public fn get(self): i64 => self.val;

    public fn inc(self) {
        self.val += 1;
    }

    pub static fn zero(): i64 => 0;   // фабричный метод
}

fn main() {
    var start = Counter::zero();   // статический вызов
    var c = new Counter(start);
    c.inc();
    c.inc();
    println!(c.get());   // 2
}
```

### Разница между обычными и статическими методами

| Вид | Синтаксис объявления | Синтаксис вызова | Доступ к `self` |
|-----|---------------------|------------------|----------------|
| Обычный метод | `public fn f(self)` | `obj.f()` | Да |
| Статический метод | `pub static fn f()` | `Type::f()` | Нет |

---

## 5.9 — Оператор конвейера `|>`

Оператор конвейера передаёт результат левого выражения как первый аргумент
правой функции. Вдохновлён Elixir и F#.

```orbitron
значение |> функция
```

эквивалентно:

```orbitron
функция(значение)
```

### Базовое использование

```orbitron
fn double(n: i64): i64 => n * 2;
fn inc(n: i64):    i64 => n + 1;
fn square(n: i64): i64 => n * n;

fn main() {
    var r1 = 3 |> double;                     // double(3) = 6
    var r2 = 3 |> double |> inc;              // inc(double(3)) = 7
    var r3 = 3 |> double |> inc |> square;    // square(inc(double(3))) = 49

    println!(r1);   // 6
    println!(r2);   // 7
    println!(r3);   // 49
}
```

### Зачем использовать конвейер?

Конвейер делает цепочки преобразований данных читаемыми слева направо.
Сравните два эквивалентных выражения:

```orbitron
// Вложенные вызовы — читать справа налево, изнутри наружу
var r1 = square(inc(double(3)));

// Конвейер — читать слева направо, данные текут естественно
var r2 = 3 |> double |> inc |> square;
```

### Конвейер с дополнительными аргументами

Когда правая функция принимает дополнительные аргументы, перечислите их в скобках:

```orbitron
fn clamp(x: i64, lo: i64, hi: i64): i64 {
    if (x < lo) { return lo; }
    if (x > hi) { return hi; }
    return x;
}

fn main() {
    // Эквивалентно: clamp(150, 0, 100)
    var r = 150 |> clamp(0, 100);
    println!(r);   // 100
}
```

### Конвейер с лямбдами

```orbitron
var result = 5
    |> |x| x * 2
    |> |x| x + 1
    |> |x| x * x;

println!(result);   // (5*2+1)^2 = 121
```

---

## 5.10 — Функции как строительные блоки

Хорошие программы разбивают сложную логику на небольшие именованные функции.
Каждая функция делает одно дело и делает его хорошо.

```orbitron
// Площадь трапеции
fn trapezoid_area(a: i64, b: i64, h: i64): i64 => (a + b) * h / 2;

// Является ли год високосным
fn is_leap(year: i64): i64 {
    if (year % 400 == 0) { return 1; }
    if (year % 100 == 0) { return 0; }
    if (year % 4   == 0) { return 1; }
    return 0;
}

// Делится ли n на d
fn divides(d: i64, n: i64): i64 => n % d == 0 ? 1 : 0;

// Простое ли число n
fn is_prime(n: i64): i64 {
    if (n < 2) { return 0; }
    mut i = 2;
    while (i * i <= n) {
        if (divides(i, n)) { return 0; }
        i += 1;
    }
    return 1;
}

fn main() {
    println!(trapezoid_area(5, 7, 4));   // 24
    println!(is_leap(2024));              // 1
    println!(is_leap(1900));              // 0
    println!(is_prime(97));               // 1
    println!(is_prime(100));              // 0
}
```

---

## 5.11 — Взаимная рекурсия

Две функции могут вызывать друг друга:

```orbitron
fn is_even(n: i64): i64 {
    if (n == 0) { return 1; }
    return is_odd(n - 1);
}

fn is_odd(n: i64): i64 {
    if (n == 0) { return 0; }
    return is_even(n - 1);
}

fn main() {
    println!(is_even(10));   // 1
    println!(is_odd(7));     // 1
    println!(is_even(3));    // 0
}
```

Благодаря проходу предварительных объявлений обе функции могут ссылаться
друг на друга независимо от порядка их объявления.

---

## 5.12 — Итоговая таблица

| Концепция | Синтаксис | Примечание |
|-----------|-----------|-----------|
| Объявить | `fn name(a: i64): i64 { }` | Аннотации типов необязательны |
| Fat-arrow тело | `fn name(a: i64): i64 => expr;` | Однострочная функция |
| Стрелка возврата | `fn name(a: i64) -> i64 { }` | Альтернативный синтаксис |
| Параметр по умолчанию | `fn f(x: i64, y: i64 = 0)` | y необязателен при вызове |
| Вызвать | `name(arg1, arg2)` | — |
| Вернуть значение | `return expr;` | Неявный return 0 если пропущен |
| Рекурсия | `fn f(n: i64) { return f(n-1); }` | Полностью поддерживается |
| Лямбда | `\|x, y\| x + y` | Анонимная функция |
| Конвейер | `x \|> f \|> g` | То же что `g(f(x))` |
| Статический метод | `pub static fn f()` | Вызов: `Type::f()` |
| Область видимости | `var`/`mut` переменные локальны | `#const` верхнего уровня доступны везде |

---

## Полный пример

```orbitron
// examples/03_functions/basics.ot

#const PI_INT: i64 = 3;   // приближение

fn square(n: i64): i64 => n * n;
fn cube(n: i64):   i64 => n * n * n;
fn abs(n: i64):    i64 => n >= 0 ? n : -n;

fn sum_up_to(n: i64): i64 {
    mut total = 0;
    for i in 1..=n { total += i; }
    return total;
}

// Параметр по умолчанию
fn power(base: i64, exp: i64 = 2): i64 {
    if (exp == 0) { return 1; }
    return base * power(base, exp - 1);
}

fn double(n: i64): i64 => n * 2;
fn inc(n: i64):    i64 => n + 1;

// Лямбда
var clamp100 = |x| x > 100 ? 100 : x;

fn main() {
    println!(square(7));       // 49
    println!(cube(3));         // 27
    println!(abs(-42));        // 42
    println!(sum_up_to(10));   // 55
    println!(power(2, 8));     // 256
    println!(power(5));        // 25 — exp = 2 по умолчанию

    // Лямбда в конвейере
    var result = 5 |> double |> inc |> square;
    println!(result);          // (5*2+1)^2 = 121

    // Лямбда напрямую
    println!(clamp100(50));    // 50
    println!(clamp100(200));   // 100
}
```

---

← [Глава 4 — Управление потоком](ch04_control_flow.md) | [Глава 6 — Коллекции →](ch06_collections.md)
