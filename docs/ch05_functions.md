# Глава 5 — Функции

Функции — фундаментальные строительные блоки любой программы на Orbitron.
Они позволяют именовать, переиспользовать и компоновать логику.

---

## 5.1 — Объявление функций

Функция объявляется с ключевым словом `func`:

```orbitron
func имя(параметр1: тип, параметр2: тип): тип_возврата {
    // тело
    return значение;
}
```

Минимальный пример:

```orbitron
func greet() {
    println("Привет из функции!");
}
```

Функция с параметрами и возвращаемым значением:

```orbitron
func add(a: int, b: int): int {
    return a + b;
}
```

### Вызов функций

```orbitron
func main() {
    greet();               // вызов без аргументов
    var sum = add(3, 4);   // вызов с аргументами, захват возвращаемого значения
    println(sum);          // 7
}
```

---

## 5.2 — Параметры и возвращаемые значения

Аннотации типов на параметрах и возвращаемом значении **необязательны**. Компилятор
пока не проверяет их строго — они служат документацией.

```orbitron
// Полностью аннотировано (рекомендуется для документации)
func multiply(a: int, b: int): int {
    return a * b;
}

// Без аннотаций (короче, тоже работает)
func multiply2(a, b) {
    return a * b;
}
```

### Возвращаемое значение

Используйте `return`, чтобы выйти из функции и вернуть значение:

```orbitron
func max_of(a: int, b: int): int {
    if (a > b) { return a; }
    return b;
}
```

Функция без инструкции `return` неявно возвращает `0`.

### Множественные return

Функция может иметь несколько инструкций `return`:

```orbitron
func classify(n: int): int {
    if (n > 0) { return 1; }   // положительное
    if (n < 0) { return -1; }  // отрицательное
    return 0;                  // ноль
}
```

---

## 5.3 — Функция main

Каждая программа должна содержать функцию `main` — это точка входа:

```orbitron
func main() {
    // программа начинается здесь
    println("Привет");
}
```

`main` не принимает параметров и ничего не возвращает (неявно возвращает 0 в ОС).

---

## 5.4 — Область видимости

### Локальные переменные

Переменные, объявленные внутри функции, **локальны** для этой функции:

```orbitron
func compute() {
    var x = 10;   // локальна для compute
    println(x);
}

func main() {
    compute();
    // println(x);   // ОШИБКА — x здесь не видна
}
```

### Глобальные константы

Константы, объявленные на верхнем уровне (вне всех функций), видны всем функциям:

```orbitron
const LIMIT: int = 100;

func check(n: int): int {
    return n < LIMIT ? 1 : 0;
}

func main() {
    println(check(50));    // 1
    println(check(200));   // 0
    println(LIMIT);        // 100
}
```

### Функции, вызывающие друг друга

Функции могут вызывать другие функции, объявленные в любом месте файла.
Порядок объявления не важен — компилятор делает проход для предварительных объявлений:

```orbitron
func main() {
    println(helper());   // ОК — helper объявлена ниже
}

func helper(): int {
    return 42;
}
```

---

## 5.5 — Рекурсия

Функция может вызывать саму себя. Orbitron полностью поддерживает рекурсию:

```orbitron
func factorial(n: int): int {
    if (n <= 1) { return 1; }
    return n * factorial(n - 1);
}

func main() {
    println(factorial(1));   // 1
    println(factorial(5));   // 120
    println(factorial(10));  // 3628800
}
```

### Числа Фибоначчи (рекурсивно)

```orbitron
func fib(n: int): int {
    if (n <= 1) { return n; }
    return fib(n - 1) + fib(n - 2);
}
```

### Числа Фибоначчи (итеративно — эффективнее)

Рекурсия может быть медленной для больших входных данных. Используйте
итерацию, когда важна производительность:

```orbitron
func fib_iter(n: int): int {
    if (n <= 1) { return n; }
    var a = 0;
    var b = 1;
    var i = 2;
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
func gcd(a: int, b: int): int {
    if (b == 0) { return a; }
    return gcd(b, a % b);
}

func main() {
    println(gcd(48, 18));    // 6
    println(gcd(100, 75));   // 25
}
```

---

## 5.6 — Оператор конвейера `|>`

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
func double(n: int): int { return n * 2; }
func inc(n: int):    int { return n + 1; }
func square(n: int): int { return n * n; }

func main() {
    var r1 = 3 |> double;                     // double(3) = 6
    var r2 = 3 |> double |> inc;              // inc(double(3)) = 7
    var r3 = 3 |> double |> inc |> square;    // square(inc(double(3))) = 49

    println(r1);   // 6
    println(r2);   // 7
    println(r3);   // 49
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
func clamp(x: int, lo: int, hi: int): int {
    if (x < lo) { return lo; }
    if (x > hi) { return hi; }
    return x;
}

func main() {
    // Эквивалентно: clamp(150, 0, 100)
    var r = 150 |> clamp(0, 100);
    println(r);   // 100
}
```

### Реальный пример использования

```orbitron
func abs_val(x: int): int { return x >= 0 ? x : -x; }
func double(x: int):  int { return x * 2; }
func clamp100(x: int): int { return x > 100 ? 100 : x; }

func main() {
    var result = -42 |> abs_val |> double |> clamp100;
    println(result);   // 84
}
```

---

## 5.7 — Функции как строительные блоки

Хорошие программы разбивают сложную логику на небольшие именованные функции.
Каждая функция делает одно дело и делает его хорошо.

```orbitron
// Площадь трапеции
func trapezoid_area(a: int, b: int, h: int): int {
    return (a + b) * h / 2;
}

// Является ли год високосным
func is_leap(year: int): int {
    if (year % 400 == 0) { return 1; }
    if (year % 100 == 0) { return 0; }
    if (year % 4   == 0) { return 1; }
    return 0;
}

// Делится ли n на d
func divides(d: int, n: int): int {
    return n % d == 0 ? 1 : 0;
}

// Простое ли число n
func is_prime(n: int): int {
    if (n < 2) { return 0; }
    var i = 2;
    while (i * i <= n) {
        if (divides(i, n)) { return 0; }
        i += 1;
    }
    return 1;
}

func main() {
    println(trapezoid_area(5, 7, 4));   // 24
    println(is_leap(2024));              // 1
    println(is_leap(1900));              // 0
    println(is_prime(97));               // 1
    println(is_prime(100));              // 0
}
```

---

## 5.8 — Взаимная рекурсия

Две функции могут вызывать друг друга:

```orbitron
func is_even(n: int): int {
    if (n == 0) { return 1; }
    return is_odd(n - 1);
}

func is_odd(n: int): int {
    if (n == 0) { return 0; }
    return is_even(n - 1);
}

func main() {
    println(is_even(10));   // 1
    println(is_odd(7));     // 1
    println(is_even(3));    // 0
}
```

Благодаря проходу предварительных объявлений обе функции могут ссылаться
друг на друга независимо от порядка их объявления.

---

## 5.9 — Итоговая таблица

| Концепция | Синтаксис | Примечание |
|-----------|-----------|-----------|
| Объявить | `func name(a: int): int { }` | Аннотации типов необязательны |
| Вызвать | `name(arg1, arg2)` | — |
| Вернуть значение | `return expr;` | Неявный return 0 если пропущен |
| Рекурсия | `func f(n) { return f(n-1); }` | Полностью поддерживается |
| Конвейер | `x \|> f \|> g` | То же что `g(f(x))` |
| Область видимости | Локальные переменные приватны | Константы верхнего уровня доступны везде |

---

## Полный пример

```orbitron
// examples/03_functions/basics.ot

const PI_INT: int = 3;   // приближение

func square(n: int): int { return n * n; }
func cube(n: int):   int { return n * n * n; }
func abs(n: int):    int { return n >= 0 ? n : -n; }

func sum_up_to(n: int): int {
    var total = 0;
    for i in 1..=n { total += i; }
    return total;
}

func power(base: int, exp: int): int {
    if (exp == 0) { return 1; }
    return base * power(base, exp - 1);
}

func double(n: int): int { return n * 2; }
func inc(n: int):    int { return n + 1; }

func main() {
    println(square(7));       // 49
    println(cube(3));         // 27
    println(abs(-42));        // 42
    println(sum_up_to(10));   // 55
    println(power(2, 8));     // 256

    // Цепочка конвейеров
    var result = 5 |> double |> inc |> square;
    println(result);          // (5*2+1)^2 = 121
}
```

---

← [Глава 4 — Управление потоком](ch04_control_flow.md) | [Глава 6 — Коллекции →](ch06_collections.md)
