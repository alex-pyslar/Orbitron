# Глава 9 — Стандартная библиотека

Стандартная библиотека Orbitron написана на самом Orbitron и расположена
в директории `stdlib/`. Она покрывает математику, побитовые операции,
алгоритмы, системные вызовы, сетевое взаимодействие и базы данных.

---

## 9.1 — Импорт модулей

Стандартные модули подключаются с префиксом `std/`:

```orbitron
import "std/math";   // математические функции
import "std/bits";   // побитовые операции
import "std/algo";   // алгоритмические утилиты
import "std/sys";    // системные вызовы Linux
import "std/net";    // TCP/UDP сетевое взаимодействие
import "std/db";     // SQLite3
```

После импорта все функции и константы модуля становятся доступны напрямую.

### Расположение библиотеки

Директория `stdlib/` должна находиться:
1. Рядом с бинарником `orbitron` (рекомендуется)
2. По пути `$ORBITRON_HOME/stdlib/`

---

## 9.2 — std/math — Математика

Математические функции и физические константы.

```orbitron
import "std/math";
```

### Константы

| Константа | Значение | Описание |
|-----------|---------|----------|
| `PI` | ≈ 3.14159... (float) | Число π |
| `E` | ≈ 2.71828... (float) | Число Эйлера |
| `INT_MAX` | 9223372036854775807 (int) | Максимальное значение int (i64) |

### Функции

| Функция | Описание |
|---------|----------|
| `abs(x)` | Абсолютное значение \|x\| |
| `max(a, b)` | Максимум из двух |
| `min(a, b)` | Минимум из двух |
| `clamp(val, lo, hi)` | Ограничение val в диапазоне [lo, hi] |
| `factorial(n)` | n! (n ≥ 0) |
| `fib(n)` | n-е число Фибоначчи (с нуля) |
| `gcd(a, b)` | Наибольший общий делитель |
| `lcm(a, b)` | Наименьшее общее кратное |
| `sum_range(a, b)` | Сумма целых от a до b включительно |
| `sign(x)` | Знак числа: -1, 0 или 1 |
| `is_prime(n)` | 1 если n простое, 0 иначе |

### Примеры

```orbitron
import "std/math";

func main() {
    // Константы
    println(PI);                    // 3.141593
    println(E);                     // 2.718282
    println(INT_MAX);               // 9223372036854775807

    // Основные функции
    println(abs(-42));              // 42
    println(max(10, 20));           // 20
    println(min(10, 20));           // 10
    println(clamp(150, 0, 100));    // 100

    // Числа и последовательности
    println(factorial(10));         // 3628800
    println(fib(10));               // 55
    println(gcd(48, 18));           // 6
    println(lcm(4, 6));             // 12
    println(sum_range(1, 100));     // 5050

    // Другое
    println(sign(-5));              // -1
    println(sign(0));               // 0
    println(sign(7));               // 1
    println(is_prime(97));          // 1
    println(is_prime(100));         // 0
}
```

---

## 9.3 — std/bits — Побитовые операции

Функции для работы на уровне битов. В Orbitron нет встроенных побитовых операторов,
поэтому вся битовая арифметика реализована через эти функции.

```orbitron
import "std/bits";
```

### Функции

| Функция | Описание |
|---------|----------|
| `bit_count(x)` | Количество установленных битов (popcount) |
| `bit_len(x)` | Длина в битах: floor(log2(x)) + 1 |
| `is_pow2(x)` | 1 если x является степенью двойки |
| `next_pow2(x)` | Следующая степень двойки ≥ x |
| `prev_pow2(x)` | Предыдущая степень двойки ≤ x |
| `low_bit(x)` | Младший установленный бит |
| `shl(x, n)` | Сдвиг влево: x * 2^n |
| `shr(x, n)` | Сдвиг вправо: x / 2^n |
| `floor_log2(x)` | Целочисленный log2 (нижнее округление) |
| `reverse_bits(x, n)` | Обратить n младших битов x |

### Примеры

```orbitron
import "std/bits";

func main() {
    // Подсчёт битов
    println(bit_count(0));       // 0
    println(bit_count(255));     // 8  (все 8 битов установлены)
    println(bit_count(1023));    // 10

    // Длина в битах
    println(bit_len(1));         // 1
    println(bit_len(8));         // 4  (1000 → 4 бита)
    println(bit_len(255));       // 8

    // Степени двойки
    println(is_pow2(16));        // 1  (16 = 2^4)
    println(is_pow2(15));        // 0
    println(next_pow2(5));       // 8  (следующая ≥ 5)
    println(next_pow2(8));       // 8  (8 уже степень двойки)
    println(prev_pow2(7));       // 4  (предыдущая ≤ 7)

    // Сдвиги
    println(shl(1, 10));         // 1024  (1 << 10)
    println(shr(1024, 3));       // 128   (1024 >> 3)

    // Логарифм
    println(floor_log2(1));      // 0
    println(floor_log2(8));      // 3
    println(floor_log2(100));    // 6

    // Низший бит
    println(low_bit(12));        // 4  (12 = 1100₂, младший бит = 100₂ = 4)
}
```

### Типичные применения

```orbitron
import "std/bits";

func main() {
    // Проверить чётность через биты
    var n = 42;
    var is_even = low_bit(n) == 1 ? 0 : 1;    // чётно если младший бит == 0

    // Следующая степень двойки для выделения буфера
    var needed = 100;
    var buf_size = next_pow2(needed);    // 128
    println(buf_size);

    // Количество итераций log2
    var iterations = floor_log2(1024);   // 10
    println(iterations);
}
```

---

## 9.4 — std/algo — Алгоритмические утилиты

Набор вспомогательных функций для сравнений, интерполяции, работы с цифрами
и числовых последовательностей.

```orbitron
import "std/algo";
```

### Функции сравнения

| Функция | Описание |
|---------|----------|
| `min3(a, b, c)` | Минимум из трёх |
| `max3(a, b, c)` | Максимум из трёх |
| `median3(a, b, c)` | Медиана из трёх |

### Интерполяция и отображение

| Функция | Описание |
|---------|----------|
| `lerp(lo, hi, t)` | Линейная интерполяция, t в диапазоне [0..100] |
| `map_range(val, in_lo, in_hi, out_lo, out_hi)` | Перемасштабирование из одного диапазона в другой |
| `dist(a, b)` | Расстояние: \|a - b\| |
| `near(a, b, tol)` | 1 если \|a-b\| <= tol |

### Цифровые операции

| Функция | Описание |
|---------|----------|
| `digit_count(x)` | Количество десятичных цифр |
| `digit_sum(x)` | Сумма цифр |
| `reverse_digits(x)` | Обратить цифры числа |
| `is_palindrome_num(x)` | 1 если число-палиндром |

### Степени и последовательности

| Функция | Описание |
|---------|----------|
| `ipow(base, exp)` | Целочисленное возведение в степень (быстрое) |
| `isqrt(n)` | Целочисленный квадратный корень (нижнее округление) |
| `is_square(n)` | 1 если n — точный квадрат |
| `triangle(n)` | Треугольное число T(n) = n*(n+1)/2 |
| `is_triangle(n)` | 1 если n — треугольное число |
| `cycle(x, delta, n)` | Циклическое смещение: (x + delta) mod n |

### Примеры

```orbitron
import "std/algo";

func main() {
    // Сравнения
    println(min3(5, 2, 8));               // 2
    println(max3(5, 2, 8));               // 8
    println(median3(5, 2, 8));            // 5

    // Интерполяция
    println(lerp(0, 100, 50));            // 50 (50% пути)
    println(lerp(0, 200, 25));            // 50 (25% пути до 200)
    println(map_range(50, 0, 100, 0, 255)); // 127 (перевод в 8-битный диапазон)

    // Цифры
    println(digit_count(12345));          // 5
    println(digit_sum(12345));            // 15 (1+2+3+4+5)
    println(reverse_digits(12345));       // 54321
    println(is_palindrome_num(12321));    // 1
    println(is_palindrome_num(12345));    // 0

    // Степени и корни
    println(ipow(2, 10));                 // 1024
    println(isqrt(144));                  // 12
    println(is_square(144));             // 1
    println(is_square(145));             // 0

    // Треугольные числа
    println(triangle(10));               // 55  (1+2+...+10)
    println(is_triangle(55));            // 1
    println(is_triangle(56));            // 0

    // Расстояние и близость
    println(dist(3, 7));                 // 4
    println(near(10, 12, 3));            // 1  (|10-12|=2 <= 3)

    // Циклическое смещение
    println(cycle(6, 1, 7));             // 0  ((6+1) mod 7)
    println(cycle(0, -1, 7));            // 6  ((0-1+7) mod 7)
}
```

---

## 9.5 — std/sys — Системные вызовы Linux

Константы номеров системных вызовов Linux (x86-64) и обёртки для наиболее
распространённых из них.

```orbitron
import "std/sys";
```

> **Только LLVM-бэкенд.** Эти функции используют прямые системные вызовы Linux
> и не работают в JVM-бэкенде.

### Константы файловых дескрипторов

| Константа | Значение | Описание |
|-----------|---------|----------|
| `STDIN` | 0 | Стандартный ввод |
| `STDOUT` | 1 | Стандартный вывод |
| `STDERR` | 2 | Стандартный вывод ошибок |

### Основные номера системных вызовов

| Константа | Значение | Системный вызов |
|-----------|---------|----------------|
| `SYS_READ` | 0 | read() |
| `SYS_WRITE` | 1 | write() |
| `SYS_OPEN` | 2 | open() |
| `SYS_CLOSE` | 3 | close() |
| `SYS_EXIT` | 60 | exit() |
| `SYS_GETPID` | 39 | getpid() |
| `SYS_FORK` | 57 | fork() |
| `SYS_MMAP` | 9 | mmap() |
| `SYS_NANOSLEEP` | 35 | nanosleep() |

### Функции-обёртки

| Функция | Описание |
|---------|----------|
| `sys_write(fd, buf, n)` | Запись n байт из buf в fd |
| `sys_read(fd, buf, n)` | Чтение n байт из fd в buf |
| `sys_open(path, flags)` | Открытие файла, возвращает fd |
| `sys_close(fd)` | Закрытие файлового дескриптора |
| `sys_exit(code)` | Завершение процесса |
| `sys_getpid()` | Получение PID текущего процесса |
| `sys_getppid()` | Получение PPID (родителя) |
| `sys_fork()` | Создание дочернего процесса |
| `sys_kill(pid, sig)` | Отправка сигнала процессу |
| `sys_alloc(size)` | Выделение памяти через mmap |
| `sys_free(addr, size)` | Освобождение памяти через munmap |
| `sys_sleep(seconds)` | Ожидание заданное количество секунд |

### Пример

```orbitron
import "std/sys";

func main() {
    // Получить PID процесса
    var pid = sys_getpid();
    println($"Мой PID: {pid}");

    // Выделить буфер и записать байты
    var buf = sys_alloc(16);
    ptr_write_byte(buf,     72);   // 'H'
    ptr_write_byte(buf + 1, 101);  // 'e'
    ptr_write_byte(buf + 2, 108);  // 'l'
    ptr_write_byte(buf + 3, 108);  // 'l'
    ptr_write_byte(buf + 4, 111);  // 'o'
    ptr_write_byte(buf + 5, 10);   // '\n'

    // Прямой системный вызов write(1, buf, 6)
    syscall(SYS_WRITE, STDOUT, buf, 6);

    sys_free(buf, 16);
}
```

---

## 9.6 — std/net — Сетевое программирование

TCP/UDP сокеты для написания сетевых клиентов и серверов.

```orbitron
import "std/net";
```

> **Только LLVM-бэкенд.** Требует POSIX-совместимой системы (Linux/macOS).

### Константы

| Константа | Значение | Описание |
|-----------|---------|----------|
| `AF_INET` | 2 | IPv4 |
| `SOCK_STREAM` | 1 | TCP |
| `SOCK_DGRAM` | 2 | UDP |
| `INADDR_ANY` | 0 | Любой интерфейс (для bind) |
| `INADDR_LOOPBACK` | 2130706433 | 127.0.0.1 |

### Функции

| Функция | Описание |
|---------|----------|
| `tcp_socket()` | Создать TCP-сокет |
| `udp_socket()` | Создать UDP-сокет |
| `net_ip(a, b, c, d)` | Упаковать IPv4-адрес из 4 октетов |
| `tcp_connect(sock, ip, port)` | Подключиться к серверу |
| `net_reuseaddr(sock)` | Установить SO_REUSEADDR |
| `net_bind(sock, ip, port)` | Привязать сокет к адресу |
| `net_listen(sock, backlog)` | Перевести сокет в режим прослушивания |
| `net_accept(sock)` | Принять входящее соединение |
| `net_send(sock, buf, n)` | Отправить n байт |
| `net_recv(sock, buf, n)` | Принять данные |
| `net_close(sock)` | Закрыть сокет |

### Пример TCP-клиента

```orbitron
import "std/net";
import "std/sys";

func main() {
    // Подключиться к localhost:8080
    var sock = tcp_socket();
    var ip   = net_ip(127, 0, 0, 1);
    var res  = tcp_connect(sock, ip, 8080);

    if (res != 0) {
        println("Ошибка подключения");
        sys_exit(1);
    }

    // Отправить сообщение
    var buf = sys_alloc(64);
    ptr_write_byte(buf,     72);   // 'H'
    ptr_write_byte(buf + 1, 105);  // 'i'
    ptr_write_byte(buf + 2, 10);   // '\n'
    net_send(sock, buf, 3);

    // Принять ответ
    var recv_buf = sys_alloc(1024);
    var n = net_recv(sock, recv_buf, 1024);
    println($"Получено байт: {n}");

    net_close(sock);
    sys_free(buf, 64);
    sys_free(recv_buf, 1024);
}
```

### Пример TCP-сервера

```orbitron
import "std/net";
import "std/sys";

func main() {
    var server = tcp_socket();
    net_reuseaddr(server);
    net_bind(server, INADDR_ANY, 8080);
    net_listen(server, 5);
    println("Сервер слушает порт 8080");

    loop {
        var client = net_accept(server);
        if (client < 0) { break; }

        var buf = sys_alloc(1024);
        var n   = net_recv(client, buf, 1024);
        net_send(client, buf, n);   // эхо

        net_close(client);
        sys_free(buf, 1024);
    }
}
```

---

## 9.7 — std/db — SQLite3

Обёртки для работы с базой данных SQLite3.

```orbitron
import "std/db";
```

> **Только LLVM-бэкенд.** При компиляции требуется `-lsqlite3`.
> Установка: `sudo apt install libsqlite3-dev`

### Константы

| Константа | Значение | Описание |
|-----------|---------|----------|
| `SQLITE_OK` | 0 | Успех |
| `SQLITE_ROW` | 100 | Есть строка для чтения |
| `SQLITE_DONE` | 101 | Запрос завершён |

### Основные функции

| Функция | Описание |
|---------|----------|
| `db_open(path)` | Открыть базу данных, возвращает дескриптор |
| `db_close(db)` | Закрыть базу данных |
| `db_exec(db, sql)` | Выполнить SQL без результатов |
| `db_prepare(db, sql)` | Подготовить SQL-запрос |
| `db_step(stmt)` | Выполнить шаг запроса (SQLITE_ROW или SQLITE_DONE) |
| `db_finalize(stmt)` | Освободить подготовленный запрос |
| `db_col_int(stmt, col)` | Получить целочисленное значение столбца |
| `db_col_count(stmt)` | Количество столбцов в результате |
| `db_last_rowid(db)` | Rowid последней вставленной строки |
| `db_changes(db)` | Количество изменённых строк |

### Пример

```orbitron
import "std/db";
import "std/sys";

func main() {
    // Открыть базу данных
    var db = db_open(cstr("data.db"));

    // Создать таблицу
    db_exec(db, cstr("CREATE TABLE IF NOT EXISTS users (id INTEGER, score INTEGER);"));

    // Вставить строки
    db_exec(db, cstr("INSERT INTO users VALUES (1, 100);"));
    db_exec(db, cstr("INSERT INTO users VALUES (2, 200);"));
    db_exec(db, cstr("INSERT INTO users VALUES (3, 150);"));

    // Прочитать данные
    var stmt = db_prepare(db, cstr("SELECT id, score FROM users ORDER BY score DESC;"));
    loop {
        var rc = db_step(stmt);
        if (rc != SQLITE_ROW) { break; }

        var id    = db_col_int(stmt, 0);
        var score = db_col_int(stmt, 1);
        println($"Пользователь {id}: {score} очков");
    }
    db_finalize(stmt);

    db_close(db);
}
```

---

## 9.8 — Написание собственных модулей

Любой файл `.ot` в директории `src/` проекта может быть импортирован:

```orbitron
import "utils";      // загружает src/utils.ot
import "net/http";   // загружает src/net/http.ot
```

Пример модуля `src/utils.ot`:

```orbitron
// src/utils.ot

const DEFAULT_TIMEOUT: int = 30;

func clamp_score(s: int): int {
    if (s < 0)   { return 0; }
    if (s > 100) { return 100; }
    return s;
}

func sign(x: int): int {
    if (x > 0) { return 1; }
    if (x < 0) { return -1; }
    return 0;
}
```

Использование в `src/main.ot`:

```orbitron
import "utils";

func main() {
    println(clamp_score(150));    // 100
    println(clamp_score(-10));    // 0
    println(sign(-5));            // -1
    println(DEFAULT_TIMEOUT);     // 30
}
```

---

## 9.9 — Текущие ограничения стандартной библиотеки

- Функции stdlib не могут принимать массивы в качестве параметров
  (все параметры — i64, массивы требуют передачи как указатель)
- Строковый тип пока отсутствует; `cstr()` создаёт указатель на C-строку
- Нет модуля для работы с файловой системой высокого уровня (используйте std/sys)
- Нет форматирования чисел / строк
- Все функции стандартной библиотеки работают только с `int` и `float`

---

← [Глава 8 — Специальные возможности](ch08_features.md) | [Глава 10 — Проекты и модули →](ch10_projects.md)
