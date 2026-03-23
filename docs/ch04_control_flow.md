# Глава 4 — Управление потоком

Управление потоком определяет порядок выполнения инструкций. Orbitron предоставляет
условные операторы, сопоставление с образцом и богатый набор циклов, вдохновлённых
несколькими языками программирования.

---

## 4.1 — if / else

Базовый условный оператор:

```orbitron
if (условие) {
    // выполняется, когда условие истинно (ненулевое)
}
```

С ветвью `else`:

```orbitron
let score = 75;

if (score >= 90) {
    println!("Отлично");
} else {
    println!("Старайся лучше");
}
```

С цепочкой `else if`:

```orbitron
let score = 75;

if (score >= 90) {
    println!("A — Отлично");
} else if (score >= 80) {
    println!("B — Хорошо");
} else if (score >= 70) {
    println!("C — Удовлетворительно");
} else if (score >= 60) {
    println!("D — Ниже среднего");
} else {
    println!("F — Не сдано");
}
```

### Правила для условия

- Условие должно быть в скобках: `if (x > 0) { ... }`
- Любое ненулевое целое число считается истинным; `0` — ложным
- Логические операторы `&&`, `||`, `!` комбинируют условия

```orbitron
let age    = 25;
let income = 50000;

if (age >= 18 && income > 30000) {
    println!("Можно оформить кредит");
}

if (age < 18 || income < 10000) {
    println!("Не подходит");
}
```

---

## 4.2 — unless

`unless` — инверсия `if`: тело выполняется, когда условие **ложно**.
Вдохновлён Ruby, в некоторых ситуациях читается более естественно:

```orbitron
let divisor = 5;

unless (divisor == 0) {
    println!(100 / divisor);   // безопасное деление
}
```

Это в точности эквивалентно `if (!divisor == 0)`, но читается лучше.

Ещё примеры:

```orbitron
let authenticated = false;

unless (authenticated) {
    println!("Пожалуйста, войдите в систему.");
}

let count = 10;
unless (count == 0) {
    println!("Обрабатываем \{count} элементов");
}
```

> **Примечание:** `unless` не поддерживает ветвь `else`. Для сложной логики
> используйте обычный `if`.

---

## 4.3 — Тернарный оператор `? :`

Тернарный оператор — компактный выбор между двумя значениями:

```orbitron
var result = условие ? значение_если_истина : значение_если_ложь;
```

Примеры:

```orbitron
let x = 10;
let abs_x = x >= 0 ? x : -x;           // абсолютное значение

let a = 7;
let b = 3;
let max = a > b ? a : b;               // максимум
let min = a < b ? a : b;               // минимум

let n = 4;
let is_even = n % 2 == 0 ? 1 : 0;     // 1 если чётное
```

### Цепочки тернарных операторов

Тернарный оператор право-ассоциативен, что позволяет строить цепочки:

```orbitron
let score = 82;
let grade = score >= 90 ? 4 :
            score >= 80 ? 3 :
            score >= 70 ? 2 :
            score >= 60 ? 1 : 0;

println!("Оценка: \{grade}");   // Оценка: 3
```

Это эквивалентно вложенным if/else, но компактнее.

---

## 4.4 — match (сопоставление с образцом)

`match` проверяет выражение против списка образцов и выполняет первую совпавшую ветвь:

```orbitron
match выражение {
    образец1 => { /* блок */ }
    образец2 => { /* блок */ }
    _        => { /* джокер: совпадает с любым значением */ }
}
```

### Сопоставление с целыми числами

```orbitron
let day = 3;

match day {
    1 => { println!("Понедельник"); }
    2 => { println!("Вторник"); }
    3 => { println!("Среда"); }
    4 => { println!("Четверг"); }
    5 => { println!("Пятница"); }
    _ => { println!("Выходной"); }
}
```

### Сопоставление с перечислениями

Наиболее распространённый случай использования `match` — перечисления:

```orbitron
enum Direction { North, South, East, West }

let dir = Direction.East;

match dir {
    Direction.North => { println!("Движение на север"); }
    Direction.South => { println!("Движение на юг"); }
    Direction.East  => { println!("Движение на восток"); }
    Direction.West  => { println!("Движение на запад"); }
}
```

### Образец-джокер `_`

Образец `_` совпадает с любым значением, не покрытым предыдущими ветвями:

```orbitron
enum Status { Active, Inactive, Banned, Deleted }

let s = Status.Deleted;

match s {
    Status.Active   => { println!("Пользователь активен"); }
    Status.Inactive => { println!("Пользователь неактивен"); }
    _               => { println!("Вход невозможен"); }
}
```

### match как выражение

`match` может использоваться как **выражение** — оно возвращает значение,
которое можно присвоить переменной или использовать напрямую:

```orbitron
enum Season { Spring, Summer, Autumn, Winter }

let season = Season.Summer;

// match как выражение — результат сразу присваивается
let temp = match season {
    Season.Spring => 15
    Season.Summer => 30
    Season.Autumn => 10
    Season.Winter => -5
};

println!("Температура: \{temp} °C");   // Температура: 30 °C
```

Это позволяет строить цепочки и избегать лишних временных переменных:

```orbitron
enum Color { Red, Green, Blue }

let code = Color.Green;

// Результат match передаётся прямо в println!
println!(match code {
    Color.Red   => 1
    Color.Green => 2
    Color.Blue  => 3
    _           => 0
});
// 2
```

### Переменные внутри ветвей match

Ветви `match` могут содержать любые инструкции, включая объявление переменных:

```orbitron
enum Season { Spring, Summer, Autumn, Winter }

let season = Season.Summer;
mut temp = 0;

match season {
    Season.Spring => { temp = 15; }
    Season.Summer => { temp = 30; }
    Season.Autumn => { temp = 10; }
    Season.Winter => { temp = -5; }
}

println!("Температура: \{temp} °C");
```

---

## 4.5 — assert и assert_eq

Orbitron предоставляет встроенные функции для проверки условий во время выполнения.
Если условие нарушено, программа немедленно завершается с сообщением об ошибке.

### assert!(условие)

Проверяет, что условие истинно (ненулевое). При нарушении — аварийное завершение:

```orbitron
let x = 42;
assert!(x > 0);        // OK — x положительное
assert!(x % 2 == 0);   // OK — x чётное
assert!(x < 100);      // OK — x меньше 100

assert!(x == 0);       // АВАРИЙНОЕ ЗАВЕРШЕНИЕ: assertion failed
```

### assert_eq!(a, b)

Проверяет, что два значения равны. При несовпадении — аварийное завершение:

```orbitron
fn add(a: i64, b: i64): i64 => a + b;

fn main() {
    assert_eq!(add(2, 3), 5);    // OK
    assert_eq!(add(0, 0), 0);    // OK
    assert_eq!(add(10, -5), 5);  // OK

    assert_eq!(add(1, 1), 3);    // АВАРИЙНОЕ ЗАВЕРШЕНИЕ: assertion failed: 2 != 3
}
```

### Использование в тестовых функциях

Аннотация `@test` в сочетании с `assert_eq!` позволяет писать самопроверяющийся код:

```orbitron
fn factorial(n: i64): i64 {
    if (n <= 1) { return 1; }
    return n * factorial(n - 1);
}

@test
fn test_factorial() {
    assert_eq!(factorial(0), 1);
    assert_eq!(factorial(1), 1);
    assert_eq!(factorial(5), 120);
    assert_eq!(factorial(10), 3628800);
    println!("factorial: все тесты прошли");
}

fn main() {
    test_factorial();
}
```

### Защитное программирование

`assert!` удобен для проверки инвариантов в начале функции:

```orbitron
fn safe_divide(a: i64, b: i64): i64 {
    assert!(b != 0);   // программа не продолжится при b == 0
    return a / b;
}

fn binary_search(arr_size: i64, target: i64): i64 {
    assert!(arr_size > 0);   // массив не должен быть пустым
    // ... поиск ...
    return -1;
}
```

---

## 4.6 — Цикл for..in (диапазоны)

Цикл `for..in` перебирает целые числа в диапазоне.

### Полуоткрытый диапазон `..`

Диапазон `a..b` включает `a`, но исключает `b` (полуоткрытый интервал `[a, b)`):

```orbitron
for i in 0..5 {
    println!(i);   // выводит 0, 1, 2, 3, 4
}
```

### Замкнутый диапазон `..=`

Диапазон `a..=b` включает обе границы (замкнутый интервал `[a, b]`):

```orbitron
for i in 1..=5 {
    println!(i);   // выводит 1, 2, 3, 4, 5
}
```

### Переменная цикла

Переменная цикла (`i` в примерах) объявляется автоматически и
является локальной для тела цикла:

```orbitron
for i in 0..10 {
    let sq = i * i;
    println!("i=\{i}, sq=\{sq}");
}
// i недоступна здесь
```

### Вычисление сумм

```orbitron
mut sum = 0;
for i in 1..=100 {
    sum += i;
}
println!(sum);   // 5050 — формула Гаусса
```

---

## 4.7 — Цикл for..in (перебор массива)

Помимо числовых диапазонов, `for..in` поддерживает прямой перебор элементов массива
(как `for x in list` в Python). Это устраняет необходимость работать с индексами вручную:

```orbitron
let nums = [10, 20, 30, 40, 50];

for x in nums {
    println!(x);
}
// 10
// 20
// 30
// 40
// 50
```

### Подсчёт суммы через перебор массива

```orbitron
let data = [3, 1, 4, 1, 5, 9, 2, 6];
mut sum = 0;

for val in data {
    sum += val;
}
println!(sum);   // 31
```

### Поиск максимума

```orbitron
let vals = [42, 17, 83, 55, 6, 99, 31];
mut max = vals[0];

for v in vals {
    if (v > max) {
        max = v;
    }
}
println!(max);   // 99
```

### Фильтрация элементов

```orbitron
let scores = [75, 90, 45, 82, 55, 91, 38];

for s in scores {
    if (s >= 80) {
        println!("Отличный балл: \{s}");
    }
}
```

### Сравнение: перебор через индекс vs прямой перебор

```orbitron
let arr = [1, 2, 3, 4, 5];

// Через индекс — нужно знать размер
for i in 0..5 {
    println!(arr[i]);
}

// Прямой перебор — компактнее и читаемее
for x in arr {
    println!(x);
}
```

Используйте **прямой перебор**, когда индекс не нужен. Используйте **индексный вариант**,
когда нужно знать позицию элемента или изменять элементы по индексу.

---

## 4.8 — Многодиапазонный for

Можно перебирать два диапазона одновременно через запятую:

```orbitron
for i in 0..3, j in 0..3 {
    println(i * 10 + j);
}
```

Это эквивалентно вложенным циклам:

```
0, 1, 2, 10, 11, 12, 20, 21, 22
```

Внутренняя переменная (`j`) проходит полный диапазон для каждого значения
внешней переменной (`i`). Удобно для операций с матрицами или сетками:

```orbitron
// Таблица умножения 3×3
for row in 1..=3, col in 1..=3 {
    let product = row * col;
    println!("  \{row}x\{col}=\{product}");
}
```

---

## 4.9 — Цикл while

Цикл `while` выполняется, пока условие истинно:

```orbitron
mut n = 10;
while (n > 0) {
    println!(n);
    n -= 1;
}
// выводит 10, 9, 8, ..., 1
```

Условие проверяется **перед** каждой итерацией. Если оно изначально ложно,
тело не выполняется ни разу:

```orbitron
let x = 0;
while (x > 0) {
    println!("никогда не выполнится");
}
```

---

## 4.10 — Цикл do..while

Цикл `do..while` выполняет тело хотя бы один раз, затем проверяет условие:

```orbitron
mut n = 0;
do {
    println!(n);
    n += 1;
} while (n < 5);
// выводит 0, 1, 2, 3, 4
```

Условие проверяется **после** каждой итерации, поэтому тело всегда
выполняется хотя бы один раз:

```orbitron
let x = 100;
do {
    println!("выполняется один раз, хотя x >= 10");
} while (x < 10);
```

---

## 4.11 — Бесконечный цикл loop

`loop` выполняется бесконечно, пока не встретится инструкция `break`:

```orbitron
mut count = 0;
loop {
    count += 1;
    if (count >= 5) { break; }
    println!(count);
}
// выводит 1, 2, 3, 4
```

Используйте `loop`, когда условие выхода сложное или проверяется внутри тела:

```orbitron
mut found = 0;
mut i = 0;
loop {
    if (i * i > 1000) {
        found = i;
        break;
    }
    i += 1;
}
println!("Первое i, где i*i > 1000: \{found}");   // 32
```

---

## 4.12 — repeat N

`repeat N` выполняет тело ровно N раз. Эквивалентен `for i in 0..N`,
но без переменной цикла (удобно, когда индекс не нужен):

```orbitron
repeat 5 {
    println!("Привет!");
}
// выводит "Привет!" 5 раз
```

```orbitron
mut counter = 0;
repeat 100 {
    counter += 1;
}
println!(counter);   // 100
```

Счётчик может быть переменной:

```orbitron
let n = 10;
repeat n {
    println!("повтор");
}
```

---

## 4.13 — break и continue

### break

`break` немедленно завершает ближайший цикл:

```orbitron
for i in 0..10 {
    if (i == 5) { break; }
    println!(i);
}
// выводит 0, 1, 2, 3, 4
```

### continue

`continue` пропускает остаток текущей итерации и переходит к следующей:

```orbitron
for i in 0..10 {
    if (i % 2 == 0) { continue; }
    println!(i);
}
// выводит 1, 3, 5, 7, 9  (только нечётные)
```

### Комбинирование break и continue

```orbitron
mut i = 0;
while (true) {
    i += 1;
    if (i % 2 == 0) { continue; }  // пропускать чётные
    if (i > 15)     { break; }     // остановиться после 15
    println!(i);
}
// выводит 1, 3, 5, 7, 9, 11, 13, 15
```

---

## 4.14 — Выбор нужного цикла

| Ситуация | Лучший цикл |
|----------|------------|
| Фиксированный диапазон целых | `for i in a..b` |
| Включая правую границу | `for i in a..=b` |
| Перебор всех элементов массива | `for x in arr` |
| Индекс не нужен | `repeat N` |
| Условие проверяется до тела | `while (cond)` |
| Условие проверяется после тела (минимум один раз) | `do { } while (cond)` |
| Сложное условие выхода | `loop { if ... { break; } }` |
| Перебор двух диапазонов | `for i in 0..m, j in 0..n` |

---

## Итоговый пример

```orbitron
// examples/02_control_flow/loops.ot

fn main() {
    // for..in — замкнутый диапазон
    mut sum1 = 0;
    for i in 1..=10 { sum1 += i; }
    println!(sum1);   // 55

    // for..in — перебор массива
    let primes = [2, 3, 5, 7, 11];
    mut sum2 = 0;
    for p in primes { sum2 += p; }
    println!(sum2);   // 28

    // while
    mut x = 16;
    while (x > 1) { x /= 2; }
    println!(x);   // 1

    // do..while
    mut n = 1;
    do { n *= 2; } while (n < 100);
    println!(n);   // 128

    // repeat
    mut ticks = 0;
    repeat 7 { ticks += 1; }
    println!(ticks);   // 7

    // loop с break
    mut p = 1;
    loop {
        p *= 3;
        if (p > 1000) { break; }
    }
    println!(p);   // 2187

    // continue
    mut odds = 0;
    for i in 1..=20 {
        if (i % 2 == 0) { continue; }
        odds += 1;
    }
    println!(odds);   // 10

    // match как выражение
    let day = 6;
    let day_name = match day {
        1 => 1   // понедельник
        2 => 2   // вторник
        3 => 3   // среда
        4 => 4   // четверг
        5 => 5   // пятница
        _ => 0   // выходной
    };
    println!(day_name);   // 0

    // assert
    assert!(sum1 == 55);
    assert_eq!(sum2, 28);
    println!("Все проверки прошли");
}
```

---

← [Глава 3 — Основы языка](ch03_basics.md) | [Глава 5 — Функции →](ch05_functions.md)
