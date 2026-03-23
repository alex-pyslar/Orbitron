# Глава 6 — Коллекции

Orbitron предоставляет несколько типов коллекций: **массивы** — упорядоченные
последовательности целых чисел, **перечисления** — именованные наборы целочисленных
констант, и **кортежи** — компактные группы значений.

---

## 6.1 — Массивы

Массив — это упорядоченная коллекция целых чисел фиксированного размера.

### Создание массива

Используйте квадратные скобки с перечислением значений через запятую:

```orbitron
var primes = [2, 3, 5, 7, 11];
var zeros  = [0, 0, 0, 0, 0, 0, 0, 0];
var data   = [100, 200, 300];
```

Элементы массива — только `int` (i64). Размер определяется при инициализации
и не может быть изменён.

### Чтение элементов

Используйте нумерацию с нуля и квадратные скобки:

```orbitron
var primes = [2, 3, 5, 7, 11];

println(primes[0]);   // 2
println(primes[1]);   // 3
println(primes[4]);   // 11
```

### Запись элементов

Элементы можно изменять через присваивание по индексу:

```orbitron
var a = [10, 20, 30];
a[0] = 99;
a[2] = a[1] * 2;   // a[2] = 40

println(a[0]);   // 99
println(a[1]);   // 20
println(a[2]);   // 40
```

### Перебор по индексу

Используйте цикл `for..in` с длиной массива:

```orbitron
var nums = [5, 10, 15, 20, 25];
for i in 0..5 {
    println(nums[i]);
}
```

### Прямой перебор элементов (`for x in arr`)

Orbitron поддерживает прямой перебор массива без явного индекса — как в Python:

```orbitron
var nums = [5, 10, 15, 20, 25];

for x in nums {
    println(x);
}
// 5, 10, 15, 20, 25
```

Это предпочтительный вариант, когда позиция элемента не важна:

```orbitron
var data = [3, 1, 4, 1, 5, 9, 2, 6];

// Подсчёт суммы через прямой перебор
var sum = 0;
for v in data {
    sum += v;
}
println(sum);   // 31

// Поиск максимума
var max = data[0];
for v in data {
    if (v > max) { max = v; }
}
println(max);   // 9
```

### Когда использовать какой способ перебора

| Ситуация | Рекомендация |
|----------|-------------|
| Нужна только значение, не позиция | `for x in arr` |
| Нужна позиция (индекс) | `for i in 0..N { arr[i] }` |
| Нужно изменить элементы | `for i in 0..N { arr[i] = ... }` |
| Перебор части массива | `for i in lo..hi { arr[i] }` |

### Вычисление суммы

```orbitron
var data = [3, 1, 4, 1, 5, 9, 2, 6];
var sum = 0;
for i in 0..8 {
    sum += data[i];
}
println(sum);   // 31
```

### Поиск максимума

```orbitron
var vals = [42, 17, 83, 55, 6, 99, 31];
var max = vals[0];
for i in 1..7 {
    if (vals[i] > max) {
        max = vals[i];
    }
}
println(max);   // 99
```

### Двумерные массивы (симуляция)

Orbitron не поддерживает многомерные массивы напрямую. Двумерную сетку можно
симулировать плоским массивом с ручным вычислением индексов:

```orbitron
// Матрица 3×3, хранится по строкам: индекс = строка*3 + столбец
var matrix = [1, 2, 3, 4, 5, 6, 7, 8, 9];

// Чтение элемента на строке 1, столбце 2
var elem = matrix[1 * 3 + 2];   // = matrix[5] = 6
println(elem);
```

### Ограничения массивов

- Элементы только `int` (массивы из float пока не поддерживаются)
- Размер фиксирован при инициализации — нет `push` или `pop`
- Нет проверки выхода за границы во время выполнения
- Нельзя передать массив в функции стандартной библиотеки (все параметры — i64)
- Нет срезов и представлений

---

## 6.2 — Практические примеры с массивами

### Сортировка пузырьком

```orbitron
// Примечание: массивы передаются в функции по ссылке (как указатель)
fn sort(arr: int, n: int) {
    for i in 0..n {
        for j in 0..n {
            if (j + 1 < n) {
                if (arr[j] > arr[j + 1]) {
                    var tmp    = arr[j];
                    arr[j]     = arr[j + 1];
                    arr[j + 1] = tmp;
                }
            }
        }
    }
}

fn main() {
    var a = [64, 34, 25, 12, 22, 11, 90];
    sort(a, 7);
    for x in a {
        println(x);
    }
    // 11 12 22 25 34 64 90
}
```

### Гистограмма оценок

```orbitron
fn main() {
    var grades  = [85, 92, 78, 95, 60, 88, 74, 91, 67, 83];
    var buckets = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];   // 10 корзин: 0-9, 10-19, ...

    for g in grades {
        var bucket = g / 10;
        buckets[bucket] += 1;
    }

    // Вывод распределения
    for i in 6..10 {
        var count = buckets[i];
        println($"  {i}0-{i}9: {count}");
    }
}
```

### Скользящее среднее

```orbitron
fn main() {
    var samples = [12, 15, 11, 18, 14, 16, 13, 17];
    var n = 8;
    var sum = 0;
    for v in samples { sum += v; }
    var avg = sum / n;
    println($"Среднее: {avg}");   // Среднее: 14
}
```

---

## 6.3 — Кортежи

Кортеж — это компактная группа из нескольких значений. Синтаксис: `(a, b)`.

### Создание кортежа

```orbitron
var point   = (10, 20);
var rgb     = (255, 128, 0);
var bounds  = (0, 100);
```

### Деструктурирующее присвоение

Чтобы извлечь значения из кортежа в отдельные переменные:

```orbitron
var (x, y) = (10, 20);
println(x);   // 10
println(y);   // 20

var (r, g, b) = (255, 128, 0);
println($"RGB: {r}, {g}, {b}");   // RGB: 255, 128, 0
```

### Параллельный обмен значений

```orbitron
var a = 5;
var b = 9;
var (a, b) = (b, a);   // обмен без временной переменной
println(a);   // 9
println(b);   // 5
```

### Кортежи как результат вычисления

```orbitron
fn main() {
    var data = [3, 1, 4, 1, 5, 9, 2, 6];

    var mn = data[0];
    var mx = data[0];
    for v in data {
        if (v < mn) { mn = v; }
        if (v > mx) { mx = v; }
    }

    // Создаём кортеж из найденных значений
    var (lo, hi) = (mn, mx);
    println($"мин={lo}, макс={hi}");   // мин=1, макс=9
}
```

### Кортежи в условиях

```orbitron
var (ok, code) = (1, 200);   // HTTP-ответ: ok=true, code=200

if (ok) {
    println($"Успех, код: {code}");
} else {
    println($"Ошибка, код: {code}");
}
```

---

## 6.4 — Перечисления (enum)

Перечисление определяет именованный набор целочисленных констант. Каждый вариант
автоматически получает значение, начиная с 0:

```orbitron
enum Color     { Red, Green, Blue }
enum Season    { Spring, Summer, Autumn, Winter }
enum Status    { Active, Inactive, Banned }
enum Direction { North, South, East, West }
```

### Значения вариантов

| Вариант | Неявное значение |
|---------|----------------|
| Первый  | 0 |
| Второй  | 1 |
| Третий  | 2 |
| …       | … |

```orbitron
enum Color { Red, Green, Blue }

var c = Color.Red;     // c == 0
var g = Color.Green;   // g == 1
var b = Color.Blue;    // b == 2

println(c);   // 0
println(g);   // 1
println(b);   // 2
```

### Перечисления с match

Перечисления наилучшим образом раскрываются при использовании с `match`:

```orbitron
enum Direction { North, South, East, West }

fn describe(d: int): int {
    match d {
        Direction.North => { println("Движение на север"); }
        Direction.South => { println("Движение на юг"); }
        Direction.East  => { println("Движение на восток"); }
        Direction.West  => { println("Движение на запад"); }
    }
    return 0;
}

fn main() {
    var dir = Direction.East;
    describe(dir);   // Движение на восток

    var other = Direction.North;
    describe(other); // Движение на север
}
```

### match как выражение с перечислением

```orbitron
enum Season { Spring, Summer, Autumn, Winter }

var season = Season.Winter;

var temp = match season {
    Season.Spring => 15
    Season.Summer => 30
    Season.Autumn => 10
    Season.Winter => -5
};

println($"Температура: {temp} °C");   // Температура: -5 °C
```

### Перечисления в условиях

Поскольку варианты enum — это просто целые числа, их можно использовать в сравнениях:

```orbitron
enum Status { Active, Inactive, Banned }

var s = Status.Active;

if (s == Status.Active) {
    println("Пользователь активен");
}

unless (s == Status.Banned) {
    println("Пользователь может войти в систему");
}
```

### Перечисления и массивы

Значения enum можно использовать как индексы массива (они — целые числа, начиная с 0):

```orbitron
enum Day { Mon, Tue, Wed, Thu, Fri, Sat, Sun }

var hours = [8, 8, 8, 8, 8, 0, 0];   // рабочие часы в каждый день

var today = Day.Wed;
var today_hours = hours[today];
println($"Рабочих часов сегодня: {today_hours}");   // 8
```

### Машины состояний

Перечисления идеально подходят для машин состояний:

```orbitron
enum Light { Red, Yellow, Green }

fn next_state(current: int): int {
    match current {
        Light.Red    => { return Light.Green;  }
        Light.Green  => { return Light.Yellow; }
        Light.Yellow => { return Light.Red;    }
        _            => { return Light.Red;    }
    }
}

fn describe_light(l: int) {
    match l {
        Light.Red    => { println("СТОП"); }
        Light.Yellow => { println("ВНИМАНИЕ"); }
        Light.Green  => { println("ЕЗЖАЙ"); }
    }
}

fn main() {
    var light = Light.Red;
    repeat 6 {
        describe_light(light);
        light = next_state(light);
    }
}
```

Вывод:
```
СТОП
ЕЗЖАЙ
ВНИМАНИЕ
СТОП
ЕЗЖАЙ
ВНИМАНИЕ
```

---

## 6.5 — Перечисления vs Константы

| Аспект | `const` | `enum` |
|--------|---------|--------|
| Группирует связанные значения | Нет | Да |
| Читаемые имена | Да | Да |
| Контроль типов | Нет | Нет |
| Работает с `match` | Частично | Отлично |
| Неявная нумерация | Нет | Да |

Используйте **константы** для единичных именованных значений. Используйте **перечисления**
для набора взаимоисключающих именованных значений.

```orbitron
// Концептуально связанные — используйте enum
enum Weekday { Mon, Tue, Wed, Thu, Fri, Sat, Sun }

// Независимые константы — используйте const
const MAX_USERS: int = 1000;
const TIMEOUT:   int = 30;
const VERSION:   int = 2;
```

---

## 6.6 — Итоговые таблицы

### Массивы

| Операция | Синтаксис |
|----------|-----------|
| Создать | `var a = [1, 2, 3];` |
| Читать | `a[i]` |
| Записать | `a[i] = value;` |
| Перебрать по индексу | `for i in 0..N { ... a[i] ... }` |
| Перебрать напрямую | `for x in a { ... x ... }` |
| Длина | Фиксирована при создании (нет `.len()`) |

### Кортежи

| Операция | Синтаксис |
|----------|-----------|
| Создать | `var t = (a, b);` |
| Деструктурировать | `var (x, y) = (a, b);` |
| Обмен | `var (a, b) = (b, a);` |

### Перечисления

| Операция | Синтаксис |
|----------|-----------|
| Объявить | `enum Name { A, B, C }` |
| Получить вариант | `Name.A` (равно 0), `Name.B` (равно 1) |
| Использовать в match | `match x { Name.A => { } _ => { } }` |
| match как выражение | `var v = match x { Name.A => 1 _ => 0 };` |
| Сравнить | `x == Name.A` |

---

## Полный пример

```orbitron
// examples/04_collections/arrays.ot

fn main() {
    var primes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29];

    // Прямой перебор — сумма
    var sum = 0;
    for p in primes { sum += p; }
    println($"Сумма первых 10 простых: {sum}");   // 129

    // Прямой перебор — максимум
    var max = primes[0];
    for p in primes {
        if (p > max) { max = p; }
    }
    println($"Наибольшее простое: {max}");   // 29

    // Прямой перебор — фильтрация
    var big = 0;
    for p in primes {
        if (p > 10) { big += 1; }
    }
    println($"Простых > 10: {big}");   // 6

    // Кортеж — деструктурирование
    var (lo, hi) = (primes[0], max);
    println($"Диапазон: {lo}..{hi}");   // Диапазон: 2..29

    // Обратный вывод через индекс
    var n = 10;
    var i = n - 1;
    while (i >= 0) {
        println(primes[i]);
        i -= 1;
    }
}
```

```orbitron
// examples/04_collections/enums.ot

enum Planet { Mercury, Venus, Earth, Mars, Jupiter, Saturn, Uranus, Neptune }

fn main() {
    var p = Planet.Earth;
    println($"Земля — планета №{p}");   // 2 (нумерация с нуля)

    // match как выражение
    var desc = match p {
        Planet.Mercury => 1
        Planet.Earth   => 2
        Planet.Mars    => 3
        _              => 0
    };
    println($"Код описания: {desc}");   // 2

    match p {
        Planet.Mercury => { println("Ближайшая к Солнцу"); }
        Planet.Earth   => { println("Наш дом"); }
        Planet.Mars    => { println("Красная планета"); }
        _              => { println("Внешняя планета"); }
    }

    // Перебор всех планет
    for i in 0..8 {
        match i {
            Planet.Mercury => { println("Меркурий"); }
            Planet.Venus   => { println("Венера");   }
            Planet.Earth   => { println("Земля");    }
            Planet.Mars    => { println("Марс");     }
            Planet.Jupiter => { println("Юпитер");   }
            Planet.Saturn  => { println("Сатурн");   }
            Planet.Uranus  => { println("Уран");     }
            Planet.Neptune => { println("Нептун");   }
        }
    }
}
```

---

← [Глава 5 — Функции](ch05_functions.md) | [Глава 7 — ООП →](ch07_oop.md)
