# Стандартная библиотека Orbitron

Стандартная библиотека (stdlib) содержит готовые функции и константы, написанные
на самом языке Orbitron. Все функции stdlib доступны через `import "std/<модуль>"`.

## Установка

Папка `stdlib/` должна находиться в одном из мест (проверяются по порядку):

1. `$ORBITRON_HOME/stdlib/` — если установлена переменная окружения
2. `{exe_dir}/stdlib/` — рядом с бинарником `orbitron`

При сборке из исходников:
```bash
cargo build --release
# stdlib/ уже лежит в корне проекта — достаточно запускать orbitron из папки проекта
```

---

## Модули

### `std/math` — математика

```orbitron
import "std/math";
```

#### Константы

| Константа   | Тип     | Значение                         |
|-------------|---------|----------------------------------|
| `PI`        | `float` | 3.14159265358979 (π)             |
| `E`         | `float` | 2.71828182845905 (e)             |
| `INT_MAX`   | `int`   | 9223372036854775807 (max i64)    |

#### Функции

| Сигнатура                          | Описание                                           |
|------------------------------------|----------------------------------------------------|
| `abs(x: int): int`                 | Абсолютное значение \|x\|                          |
| `max(a: int, b: int): int`         | Максимум из двух чисел                             |
| `min(a: int, b: int): int`         | Минимум из двух чисел                              |
| `clamp(val: int, lo: int, hi: int): int` | Ограничить val диапазоном [lo, hi]           |
| `factorial(n: int): int`           | n! — факториал (n >= 0)                            |
| `fib(n: int): int`                 | n-е число Фибоначчи (fib(0)=0, fib(1)=1)          |
| `gcd(a: int, b: int): int`         | Наибольший общий делитель (алгоритм Евклида)       |
| `lcm(a: int, b: int): int`         | Наименьшее общее кратное                           |
| `sum_range(a: int, b: int): int`   | Сумма целых от a до b включительно                 |
| `sign(x: int): int`                | Знак числа: -1, 0 или 1                            |
| `is_prime(n: int): int`            | 1 если n — простое, 0 иначе (n > 1)                |

#### Примеры

```orbitron
import "std/math";

func main() {
    println(abs(-7));          // 7
    println(max(10, 20));      // 20
    println(clamp(150, 0, 100)); // 100
    println(factorial(10));    // 3628800
    println(fib(10));          // 55
    println(gcd(12, 18));      // 6
    println(lcm(4, 6));        // 12
    println(sum_range(1, 100)); // 5050
    println(sign(-42));        // -1
    println(is_prime(97));     // 1
    println(is_prime(100));    // 0
}
```

---

### `std/bits` — битовые операции

```orbitron
import "std/bits";
```

Все функции работают с целочисленными значениями (`int` = i64).
Битовые операции реализованы через арифметику (без использования инструкций
сдвига/AND напрямую), поэтому совместимы с обоими бэкендами (LLVM и JVM).

#### Функции

| Сигнатура                          | Описание |
|------------------------------------|----------|
| `bit_count(x: int): int`           | Количество установленных битов (popcount) |
| `bit_len(x: int): int`             | Длина числа в битах: `floor(log2(x)) + 1` для x>0; 0 для x<=0 |
| `is_pow2(x: int): int`             | 1 если x — степень двойки (x > 0) |
| `next_pow2(x: int): int`           | Следующая степень двойки >= x (x >= 1) |
| `prev_pow2(x: int): int`           | Предыдущая степень двойки <= x (x >= 1) |
| `low_bit(x: int): int`             | Наименьший установленный бит; 0 если x == 0 |
| `shl(x: int, n: int): int`         | Логический сдвиг влево: x * 2^n |
| `shr(x: int, n: int): int`         | Логический сдвиг вправо: x / 2^n |
| `floor_log2(x: int): int`          | Целочисленный log2 (floor): кол-во раз делить x на 2 до 1 |
| `reverse_bits(x: int, bits: int): int` | Обратить `bits` младших битов числа |

#### Примеры

```orbitron
import "std/bits";

func main() {
    // popcount
    println(bit_count(255));   // 8  (0b11111111)
    println(bit_count(7));     // 3  (0b111)

    // длина числа в битах
    println(bit_len(256));     // 9  (0b100000000)
    println(bit_len(255));     // 8

    // степень двойки
    println(is_pow2(8));       // 1
    println(is_pow2(9));       // 0
    println(next_pow2(5));     // 8
    println(prev_pow2(9));     // 8

    // сдвиги
    println(shl(1, 10));       // 1024
    println(shr(1024, 3));     // 128

    // log2
    println(floor_log2(100));  // 6  (2^6=64 <= 100 < 128=2^7)

    // обращение битов
    println(reverse_bits(11, 4)); // 13  (0b1011 -> 0b1101)
}
```

#### Практика: выравнивание по степени двойки

```orbitron
import "std/bits";

func main() {
    var size = 300;
    var aligned = next_pow2(size);
    println(aligned);   // 512
}
```

---

### `std/algo` — алгоритмы

```orbitron
import "std/algo";
```

#### Функции

##### Трёхзначные сравнения

| Сигнатура | Описание |
|-----------|----------|
| `min3(a: int, b: int, c: int): int` | Минимум из трёх |
| `max3(a: int, b: int, c: int): int` | Максимум из трёх |
| `median3(a: int, b: int, c: int): int` | Медиана из трёх |

##### Интерполяция и перевод диапазонов

| Сигнатура | Описание |
|-----------|----------|
| `lerp(lo: int, hi: int, t: int): int` | Линейная интерполяция: lo + t*(hi-lo)/100; t в [0..100] |
| `map_range(val, in_lo, in_hi, out_lo, out_hi: int): int` | Перевод значения из одного диапазона в другой |

##### Расстояния

| Сигнатура | Описание |
|-----------|----------|
| `dist(a: int, b: int): int` | Расстояние: \|a - b\| |

##### Цифры числа

| Сигнатура | Описание |
|-----------|----------|
| `digit_count(x: int): int` | Количество цифр в |x| (min 1) |
| `digit_sum(x: int): int` | Сумма цифр |x| |
| `reverse_digits(x: int): int` | Разворот цифр: reverse_digits(1234) = 4321 |
| `is_palindrome_num(x: int): int` | 1 если число — палиндром |

##### Степени и последовательности

| Сигнатура | Описание |
|-----------|----------|
| `ipow(base: int, exp: int): int` | Быстрое целочисленное возведение в степень |
| `triangle(n: int): int` | Треугольное число T(n) = n*(n+1)/2 |
| `is_triangle(n: int): int` | 1 если n — треугольное число |
| `isqrt(n: int): int` | Целочисленный квадратный корень: floor(sqrt(n)) |
| `is_square(n: int): int` | 1 если n — точный квадрат |

##### Прочее

| Сигнатура | Описание |
|-----------|----------|
| `near(a: int, b: int, tolerance: int): int` | 1 если \|a-b\| <= tolerance |
| `cycle(x: int, delta: int, n: int): int` | Циклическое смещение (x+delta) mod n, результат >= 0 |

#### Примеры

```orbitron
import "std/algo";

func main() {
    // трёхзначные сравнения
    println(min3(7, 2, 9));     // 2
    println(max3(7, 2, 9));     // 9
    println(median3(7, 2, 9));  // 7

    // интерполяция: 75% от [0..255] → 191
    println(lerp(0, 255, 75));  // 191

    // ADC [0..1023] → яркость [0..255]
    println(map_range(512, 0, 1023, 0, 255)); // 127

    // цифры
    println(digit_sum(1234));          // 10
    println(reverse_digits(1234));     // 4321
    println(is_palindrome_num(12321)); // 1

    // степени
    println(ipow(2, 16));   // 65536
    println(isqrt(144));    // 12
    println(is_square(49)); // 1

    // треугольные числа
    println(triangle(5));   // 15
    println(is_triangle(15)); // 1

    // циклический счётчик дней (0-6)
    var day = 5;
    println(cycle(day, 3, 7)); // 1  (5+3 = 8 mod 7 = 1)
}
```

---

## Комбинирование модулей

Можно импортировать несколько модулей одновременно:

```orbitron
import "std/math";
import "std/bits";
import "std/algo";

func main() {
    // Найти ближайшую степень двойки к 2^n для факториала
    var f = factorial(5);       // 120  (из std/math)
    var p = next_pow2(f);       // 128  (из std/bits)
    var s = isqrt(p);           // 11   (из std/algo)
    println(s);
}
```

---

## Написание собственных stdlib-модулей

Создайте файл `stdlib/mymodule.ot` с обычным Orbitron-кодом:

```orbitron
// stdlib/mymodule.ot
const MY_CONST: int = 42;

func my_func(x: int): int {
    return x * MY_CONST;
}
```

Подключите в своём проекте:

```orbitron
import "std/mymodule";

func main() {
    println(my_func(2));  // 84
}
```

---

## Ограничения текущей версии

- Массивы нельзя передавать в функции stdlib (все параметры — `int` или `float`)
- Строки доступны только в `println()` — строковых функций в stdlib нет
- Все константы и функции попадают в глобальное пространство имён
  (конфликт имён при импорте нескольких модулей с одинаковыми идентификаторами)
