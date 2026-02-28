# Примеры программ на языке Orbitron

Все примеры доступны в каталоге `examples/`.

Сборка и запуск:

```bash
# имя бинарника выводится автоматически из имени файла
orbitron examples/hello.ot && ./hello

# явное имя выходного файла
orbitron -o hello examples/hello.ot && ./hello
```

---

## 1. Привет, мир (`examples/hello.ot`)

Минимальная программа: вывод строки и простая арифметика.

```orbitron
func main() {
    println("Привет, мир!");

    var x = 42;
    println(x);          // 42

    var pi: float = 3.14159;
    println(pi);         // 3.14159

    var summa = x + 8;
    println(summa);      // 50
}
```

**Ключевые концепции:**
- `println(...)` — вывод значения любого типа
- `var` — объявление переменной
- Аннотации типов (`float`) опциональны

---

## 2. Числа Фибоначчи (`examples/fibonacci.ot`)

Два подхода: рекурсия и итерация.

```orbitron
// Рекурсия — элегантна, но медленна при больших n
func fib_rec(n: int): int {
    if (n <= 1) { return n; }
    return fib_rec(n - 1) + fib_rec(n - 2);
}

// Итерация — O(n) по времени, O(1) по памяти
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

func main() {
    // Вывести 16 чисел Фибоначчи итерацией
    for i in 0..=15 {
        println(fib_iter(i));
    }
    // 0 1 1 2 3 5 8 13 21 34 55 89 144 233 377 610
}
```

**Ключевые концепции:**
- Рекурсивные функции
- Цикл `while` с счётчиком
- Цикл `for i in 0..=15` (включительный диапазон)

---

## 3. Структуры в стиле Go/Rust (`examples/oop_struct.ot`)

Геометрические вычисления с `struct` и `impl`.

```orbitron
struct Point {
    x: int,
    y: int,
}

impl Point {
    pub func dist_sq(self): int {
        return self.x * self.x + self.y * self.y;
    }

    pub func move_by(self, dx: int, dy: int) {
        self.x = self.x + dx;
        self.y = self.y + dy;
    }

    pub func scale(self, factor: int) {
        self.x = self.x * factor;
        self.y = self.y * factor;
    }
}

func main() {
    var p = Point { x: 3, y: 4 }; // литерал без new
    println(p.dist_sq());   // 25 = 3²+4²

    p.move_by(1, -1);
    println(p.x);  // 4
    println(p.y);  // 3

    // Таблица 3×3 квадратов расстояний
    for i in 0..3, j in 0..3 {
        var pt = Point { x: i, y: j };
        println(pt.dist_sq());
    }
    // 0 1 4 1 2 5 4 5 8
}
```

**Ключевые концепции:**
- Литерал структуры `Name { field: val }` без ключевого слова `new`
- Методы в `impl` блоке с явным `self`
- Вложенный цикл `for i in ..., j in ...`

---

## 4. Классы в стиле Java/C# (`examples/oop_class.ot`)

Инкапсуляция с `class`, конструктором `init` и приватными полями.

```orbitron
class BankAccount {
    private balance: int,

    init(initial: int) {
        self.balance = initial;
    }

    pub func deposit(self, amount: int) {
        if (amount > 0) {
            self.balance = self.balance + amount;
        }
    }

    pub func withdraw(self, amount: int): int {
        if (amount > 0) {
            if (self.balance >= amount) {
                self.balance = self.balance - amount;
                return 1; // успех
            }
        }
        return 0; // недостаточно средств
    }

    pub func balance(self): int {
        return self.balance;
    }
}

func main() {
    var acc = new BankAccount(500);
    acc.deposit(200);
    println(acc.balance()); // 700

    var ok = acc.withdraw(300);
    println(ok);            // 1 (успех)
    println(acc.balance()); // 400

    var fail = acc.withdraw(1000);
    println(fail);          // 0 (отказ)
    println(acc.balance()); // 400
}
```

**Ключевые концепции:**
- `class` с `private` полями
- `init(params)` — конструктор (без явного `self`)
- `new ClassName(args)` — создание объекта
- Методы возвращают значения (`return 1` / `return 0` вместо bool)

---

## 5. Ввод из консоли (`examples/input_demo.ot`)

Чтение данных от пользователя и арифметика.

```orbitron
func main() {
    println("Введите два целых числа:");
    var a = readInt();
    var b = readInt();

    println("Сумма:");      println(a + b);
    println("Произведение:"); println(a * b);

    // Классификация первого числа
    match a {
        0 => { println("a равно нулю"); }
        1 => { println("a равно единице"); }
        _ => { println("a — другое число"); }
    }

    println("Введите вещественное число:");
    var f = readFloat();
    println(f * f);   // квадрат
}
```

**Ключевые концепции:**
- `readInt()` — ввод целого (аналог `scanf("%lld")`)
- `readFloat()` — ввод дробного (аналог `scanf("%lf")`)
- `match` для классификации

---

## Сводка: когда какой стиль ООП использовать

| Ситуация                              | Рекомендация              |
|---------------------------------------|---------------------------|
| Данные + вычисления, без состояния    | `struct + impl`           |
| Инкапсулированное изменяемое состояние| `class + init`            |
| Простая геометрия, физика, математика | `struct + impl`           |
| Банковский счёт, таймер, очередь      | `class + init`            |

Оба стиля генерируют идентичный LLVM IR — разница только в синтаксисе.

---

## Полный пример: комбо всех возможностей

```orbitron
func factorial(n: int): int {
    if (n <= 1) { return 1; }
    return n * factorial(n - 1);
}

class Accumulator {
    private total: int,
    init() { self.total = 0; }
    pub func add(self, v: int) { self.total = self.total + v; }
    pub func get(self): int   { return self.total; }
}

func main() {
    var acc = new Accumulator();
    for k in 1..=5 {
        acc.add(factorial(k));
    }
    println(acc.get()); // 1+2+6+24+120 = 153

    var score = readInt();
    match score {
        1 => { println("Бронза"); }
        2 => { println("Серебро"); }
        3 => { println("Золото"); }
        _ => { println("Нет медали"); }
    }
}
```

Сборка: `orbitron -o demo examples/...`
Запуск: `echo "2" | ./demo` → выведет `153`, затем `Серебро`.
