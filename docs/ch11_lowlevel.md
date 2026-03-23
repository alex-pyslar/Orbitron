# Глава 11 — Низкоуровневое программирование

Orbitron предоставляет прямой доступ к аппаратным и системным ресурсам:
указатели, системные вызовы Linux, внешние C-функции и работу с памятью.
Эти возможности доступны только в LLVM-бэкенде.

---

## 11.1 — Указатели

### Взятие адреса переменной: `&x`

Оператор `&` возвращает адрес переменной в памяти (как целое число i64):

```orbitron
var x = 42;
var addr = &x;
println(addr);   // например: 140723456789012 (адрес в стеке)
```

Технически: компилятор создаёт `alloca` для переменной, затем
`ptrtoint` преобразует указатель в i64.

### Разыменование: `*x`

Оператор `*` читает 64-битное значение по адресу:

```orbitron
var x = 42;
var addr = &x;
var val  = *addr;    // читает x по его адресу
println(val);         // 42
```

### ptr_write — запись по адресу

```orbitron
ptr_write(addr, value);
```

Сохраняет i64 `value` по адресу `addr`:

```orbitron
var x = 0;
var addr = &x;
ptr_write(addr, 99);
println(x);   // 99
```

### ptr_write_byte — запись одного байта

```orbitron
ptr_write_byte(addr, value);
```

Сохраняет младший байт `value` по адресу `addr`. Используется для побайтовой
записи в буферы:

```orbitron
import "std/sys";

fn main() {
    var buf = sys_alloc(8);
    ptr_write_byte(buf,     72);   // 'H'
    ptr_write_byte(buf + 1, 101);  // 'e'
    ptr_write_byte(buf + 2, 108);  // 'l'
    ptr_write_byte(buf + 3, 108);  // 'l'
    ptr_write_byte(buf + 4, 111);  // 'o'
    ptr_write_byte(buf + 5, 10);   // '\n'

    syscall(SYS_WRITE, STDOUT, buf, 6);
    sys_free(buf, 8);
}
```

### ptr_read — чтение по адресу

```orbitron
var val = ptr_read(addr);
```

Аналог `*addr`. Читает i64 по адресу:

```orbitron
var x = 123;
var addr = &x;
var copy = ptr_read(addr);
println(copy);   // 123
```

### cstr — C-строка

```orbitron
var ptr = cstr("текст");
```

Создаёт нуль-терминированную C-строку и возвращает её адрес как i64:

```orbitron
import "std/sys";

fn main() {
    var msg = cstr("Привет из Orbitron!\n");
    syscall(SYS_WRITE, STDOUT, msg, 20);
}
```

### sign_ext — расширение знака

Некоторые системные функции (например, `socket`) возвращают 32-битный `int`,
но в Orbitron всё — 64-битное i64. При возврате `-1` получается `4294967295`
(0xFFFFFFFF). `sign_ext` исправляет это:

```orbitron
var raw = some_c_func();       // может вернуть 4294967295 вместо -1
var val = sign_ext(raw);       // правильный -1
```

---

## 11.2 — Управление памятью

В Orbitron нет автоматической сборки мусора. Память управляется вручную
через `std/sys`:

```orbitron
import "std/sys";

fn main() {
    // Выделить 1024 байт
    var buf = sys_alloc(1024);

    // ... использовать buf ...

    // Освободить
    sys_free(buf, 1024);
}
```

### sys_alloc

Выделяет память через `mmap(MAP_ANONYMOUS)`. Возвращает адрес начала блока:

```orbitron
var buf = sys_alloc(size_in_bytes);
```

### sys_free

Освобождает блок памяти через `munmap`:

```orbitron
sys_free(buf, size_in_bytes);
```

> **Важно:** Размер при освобождении должен совпадать с размером при выделении.

### Пример: динамический массив

```orbitron
import "std/sys";

fn main() {
    var n    = 10;
    var size = n * 8;   // 10 элементов по 8 байт (i64)
    var arr  = sys_alloc(size);

    // Заполнить квадратами
    for i in 0..n {
        ptr_write(arr + i * 8, i * i);
    }

    // Прочитать и вывести
    for i in 0..n {
        var val = ptr_read(arr + i * 8);
        println(val);
    }

    sys_free(arr, size);
}
```

---

## 11.3 — Прямые системные вызовы

Встроенная функция `syscall` вызывает системный вызов Linux напрямую:

```orbitron
syscall(номер, арг0, арг1, арг2, арг3, арг4, арг5);
```

Принимает от 1 до 7 аргументов (номер + до 6 аргументов).

### Пример: write(2)

```orbitron
import "std/sys";

fn main() {
    var msg = cstr("Hello syscall!\n");
    syscall(SYS_WRITE, STDOUT, msg, 15);
}
```

### Пример: exit(60)

```orbitron
import "std/sys";

fn main() {
    println("Выход через syscall");
    syscall(SYS_EXIT, 0);
}
```

### Пример: getpid(39)

```orbitron
import "std/sys";

fn main() {
    var pid = syscall(SYS_GETPID);
    println($"Мой PID: {pid}");
}
```

### Таблица основных системных вызовов x86-64

| Номер | Имя | Аргументы | Описание |
|-------|-----|-----------|----------|
| 0 | read | fd, buf, count | Чтение из файла |
| 1 | write | fd, buf, count | Запись в файл |
| 2 | open | path, flags | Открытие файла |
| 3 | close | fd | Закрытие файла |
| 9 | mmap | addr, len, prot, flags, fd, off | Отображение памяти |
| 11 | munmap | addr, len | Освобождение отображения |
| 35 | nanosleep | timespec_ptr, rem_ptr | Задержка |
| 39 | getpid | — | Получить PID |
| 57 | fork | — | Создать дочерний процесс |
| 60 | exit | status | Завершить процесс |
| 62 | kill | pid, sig | Отправить сигнал |

---

## 11.4 — Внешние C-функции

`extern fn` позволяет объявить и вызвать любую функцию из C-библиотек:

```orbitron
extern fn имя(param1: тип, param2: тип, ...): тип_возврата;
```

После объявления функцию можно вызывать как обычную:

```orbitron
extern fn malloc(size: int): int;
extern fn free(ptr: int);
extern fn printf(fmt: int, ...): int;

fn main() {
    var buf = malloc(64);
    // ... использовать buf ...
    free(buf);
}
```

### Вариадические функции

Добавьте `...` в список параметров для функций с переменным числом аргументов:

```orbitron
extern fn printf(fmt: int, ...): int;
extern fn sprintf(buf: int, fmt: int, ...): int;
```

### Пример: файловый ввод-вывод через libc

```orbitron
extern fn open(path: int, flags: int, mode: int): int;
extern fn read(fd: int, buf: int, n: int): int;
extern fn write(fd: int, buf: int, n: int): int;
extern fn close(fd: int): int;

import "std/sys";

const O_RDONLY: int = 0;
const O_WRONLY: int = 1;
const O_CREAT:  int = 64;
const O_TRUNC:  int = 512;

fn main() {
    // Открыть файл для записи
    var path = cstr("/tmp/test.txt");
    var fd   = open(path, O_WRONLY + O_CREAT + O_TRUNC, 420);

    if (fd < 0) {
        println("Ошибка открытия файла");
        sys_exit(1);
    }

    // Записать данные
    var msg = cstr("Привет, файл!\n");
    write(fd, msg, 15);

    close(fd);
    println("Файл записан");
}
```

### Пример: gettimeofday

```orbitron
extern fn gettimeofday(tv: int, tz: int): int;
import "std/sys";

fn main() {
    var timeval = sys_alloc(16);   // struct timeval: tv_sec (8) + tv_usec (8)
    gettimeofday(timeval, 0);

    var sec  = ptr_read(timeval);
    var usec = ptr_read(timeval + 8);
    println($"Секунды: {sec}");
    println($"Микросекунды: {usec}");

    sys_free(timeval, 16);
}
```

---

## 11.5 — Низкоуровневое сетевое программирование

При необходимости можно работать с сокетами напрямую через extern fn:

```orbitron
extern fn socket(domain: int, type: int, proto: int): int;
extern fn bind(fd: int, addr: int, addrlen: int): int;
extern fn listen(fd: int, backlog: int): int;
extern fn accept(fd: int, addr: int, addrlen: int): int;
extern fn connect(fd: int, addr: int, addrlen: int): int;
extern fn send(fd: int, buf: int, n: int, flags: int): int;
extern fn recv(fd: int, buf: int, n: int, flags: int): int;
extern fn close(fd: int): int;
extern fn setsockopt(fd: int, level: int, opt: int, val: int, vlen: int): int;

import "std/sys";

const AF_INET:     int = 2;
const SOCK_STREAM: int = 1;

fn make_sockaddr(ip: int, port: int): int {
    // struct sockaddr_in: sin_family(2) + sin_port(2) + sin_addr(4) + pad(8)
    var sa = sys_alloc(16);

    // sin_family = AF_INET (2)
    ptr_write_byte(sa, 2);
    ptr_write_byte(sa + 1, 0);

    // sin_port (big-endian)
    ptr_write_byte(sa + 2, (port / 256) % 256);
    ptr_write_byte(sa + 3, port % 256);

    // sin_addr
    ptr_write_byte(sa + 4, (ip / 16777216) % 256);
    ptr_write_byte(sa + 5, (ip / 65536) % 256);
    ptr_write_byte(sa + 6, (ip / 256) % 256);
    ptr_write_byte(sa + 7, ip % 256);

    return sa;
}

fn main() {
    var sock = socket(AF_INET, SOCK_STREAM, 0);
    var sa   = make_sockaddr(2130706433, 8080);   // 127.0.0.1:8080

    var r = connect(sock, sa, 16);
    if (r == 0) {
        println("Подключено!");
        var msg = cstr("GET / HTTP/1.0\r\n\r\n");
        send(sock, msg, 18, 0);
        close(sock);
    } else {
        println("Ошибка подключения");
    }

    sys_free(sa, 16);
}
```

Для большинства задач рекомендуется использовать `std/net` —
она скрывает эти детали за удобным API.

---

## 11.6 — Арифметика указателей

Поскольку указатели — это просто i64, над ними можно выполнять арифметические
операции:

```orbitron
import "std/sys";

fn main() {
    var buf = sys_alloc(40);   // 5 элементов по 8 байт

    // Запись через арифметику указателей
    for i in 0..5 {
        ptr_write(buf + i * 8, i * 100);
    }

    // Чтение
    for i in 0..5 {
        var val = ptr_read(buf + i * 8);
        println(val);   // 0, 100, 200, 300, 400
    }

    sys_free(buf, 40);
}
```

> **Каждый элемент i64 занимает 8 байт.** При обходе массива i64
> умножайте индекс на 8.

---

## 11.7 — Ограничения и предостережения

| Аспект | Примечание |
|--------|-----------|
| **Только LLVM** | Указатели, syscall и extern не работают в JVM-бэкенде |
| **Нет защиты** | Выход за пределы буфера — неопределённое поведение |
| **Ручная память** | `sys_alloc` нужно освобождать через `sys_free` |
| **sign_ext** | Используйте для libc-функций, возвращающих int (32-бит) |
| **Только Linux** | syscall-номера специфичны для x86-64 Linux |
| **Размер указателя** | Все адреса — 64-бит (i64), даже если значение меньше |

---

## 11.8 — Полный пример: выделение памяти и побайтовая запись

```orbitron
// examples/07_advanced/syscall_demo.ot

import "std/sys";

fn write_str(buf: int, s_arr: int, n: int) {
    for i in 0..n {
        var b = ptr_read(s_arr + i * 8);
        ptr_write_byte(buf + i, b);
    }
}

fn main() {
    println("--- Демо указателей и syscall ---");

    // Адрес переменной
    var x = 42;
    var addr = &x;
    println($"x = {x}, адрес = {addr}");

    // Запись по адресу
    ptr_write(addr, 99);
    println($"После ptr_write: x = {x}");

    // Выделение буфера и запись байтов
    var buf = sys_alloc(32);
    var bytes = [79, 114, 98, 105, 116, 114, 111, 110, 10];  // "Orbitron\n"

    for i in 0..9 {
        ptr_write_byte(buf + i, bytes[i]);
    }

    // Прямой системный вызов write
    syscall(SYS_WRITE, STDOUT, buf, 9);

    // PID процесса
    var pid = sys_getpid();
    println($"PID: {pid}");

    sys_free(buf, 32);
    println("--- Готово ---");
}
```

---

← [Глава 10 — Проекты и модули](ch10_projects.md) | [Глава 12 — Бэкенды компиляции →](ch12_backends.md)
