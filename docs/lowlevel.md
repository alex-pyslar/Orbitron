# Низкоуровневое программирование в Orbitron

Orbitron поддерживает прямой доступ к системным вызовам Linux, работу с указателями,
сетевое программирование и взаимодействие с базами данных через стандартную библиотеку.

---

## 1. Указатели и ссылки

### Получение адреса переменной — `&`

Оператор `&` возвращает адрес переменной как `int` (i64 на 64-битной платформе).

```orbitron
var x = 42;
var addr = &x;     // addr содержит адрес переменной x
println(addr);     // например, 140732834471960
```

Поддерживается для переменных типов `int`, `float`, а также массивов (возвращает адрес первого элемента).

### Разыменование указателя — `*`

Оператор `*` загружает значение типа `int` (i64) из адреса.

```orbitron
var x = 100;
var p = &x;
var val = *p;    // val == 100
```

### Запись по адресу — `ptr_write(addr, val)`

Встроенная функция `ptr_write` сохраняет `val` (i64) по адресу `addr`.

```orbitron
var buf = [0, 0, 0];
ptr_write(&buf, 99);    // buf[0] = 99
println(*(&buf));       // 99
```

### Запись байта — `ptr_write_byte(addr, val)`

Записывает один байт (младшие 8 бит `val`) по адресу `addr`.

```orbitron
var buf = sys_alloc(64);   // выделить 64 байта
ptr_write_byte(buf,     72);  // 'H'
ptr_write_byte(buf + 1, 105); // 'i'
ptr_write_byte(buf + 2, 0);   // null-terminator
```

### Чтение по адресу — `ptr_read(addr)`

Идентичен `*addr`, явная форма для читаемости.

```orbitron
var v = ptr_read(some_ptr);
```

### Адрес C-строкового литерала — `cstr("...")`

Возвращает адрес нуль-терминированной строки, хранимой как глобальная константа LLVM.
Используется для передачи строк в C-функции.

```orbitron
var path = cstr("/tmp/data.db");
var fd = sys_open(path, O_RDONLY, 0);
```

---

## 2. Системные вызовы

### Встроенная функция `syscall`

```
syscall(nr: int, a0, a1, a2, a3, a4, a5): int
```

Выполняет Linux syscall номер `nr` с аргументами `a0`–`a5`.
Возвращает значение ядра (отрицательное = код ошибки).

```orbitron
// Вывод "Hello!\n" через SYS_WRITE напрямую
var msg = cstr("Hello!\n");
syscall(1, 1, msg, 7);   // write(STDOUT, msg, 7)
```

### Стандартная библиотека: `std/sys`

```orbitron
import "std/sys";

func main() {
    var pid = sys_getpid();
    println(pid);

    var mem = sys_alloc(4096);     // выделить 4 КБ
    ptr_write(mem, 12345);
    println(*mem);
    sys_free(mem, 4096);

    sys_exit(0);
}
```

Доступные константы и функции:

| Константа/функция       | Описание                                      |
|-------------------------|-----------------------------------------------|
| `STDIN`, `STDOUT`, `STDERR` | Стандартные файловые дескрипторы          |
| `SYS_READ`, `SYS_WRITE`, ... | Номера syscall (x86-64 Linux)           |
| `O_RDONLY`, `O_WRONLY`, `O_CREAT`, ... | Флаги open()                |
| `PROT_READ`, `PROT_WRITE`, `MAP_*` | Флаги mmap()              |
| `sys_write(fd, buf, len)` | Обёртка write()                             |
| `sys_read(fd, buf, len)`  | Обёртка read()                              |
| `sys_open(path, flags, mode)` | Обёртка open()                          |
| `sys_close(fd)`           | Обёртка close()                             |
| `sys_exit(code)`          | Завершение процесса                         |
| `sys_getpid()`            | PID текущего процесса                       |
| `sys_fork()`              | fork()                                      |
| `sys_kill(pid, sig)`      | Отправка сигнала                            |
| `sys_alloc(len)`          | Выделение памяти (mmap anonymous)           |
| `sys_free(addr, len)`     | Освобождение памяти (munmap)                |
| `sys_sleep(secs)`         | Пауза в секундах                            |

---

## 3. Объявление внешних C-функций — `extern func`

Синтаксис позволяет объявить любую функцию из libc или другой C-библиотеки:

```orbitron
extern func strlen(s: int): int;
extern func malloc(size: int): int;
extern func free(ptr: int): int;
extern func memcpy(dst: int, src: int, n: int): int;
extern func printf(fmt: int, ...): int;
```

После объявления функция вызывается как обычная:

```orbitron
var len = strlen(cstr("Hello"));
println(len);   // 5

var buf = malloc(128);
memcpy(buf, cstr("World"), 5);
printf(cstr("buf = %s\n"), buf);
free(buf);
```

> **Важно**: все параметры и возвращаемое значение трактуются как `i64`.
> Это работает на 64-битных системах, где sizeof(pointer) == 8.

### Variadic функции

Добавьте `...` как последний параметр для variadic:

```orbitron
extern func dprintf(fd: int, fmt: int, ...): int;

dprintf(2, cstr("Error: code %lld\n"), 42);
```

---

## 4. Сетевое программирование — `std/net`

```orbitron
import "std/net";
import "std/sys";

func main() {
    // Создать TCP-сокет
    var fd = tcp_socket();
    if (fd < 0) {
        println(-1);
        return;
    }

    // Подключиться к 127.0.0.1:8080
    var ip = net_ip(127, 0, 0, 1);
    var rc = tcp_connect(fd, ip, 8080);
    if (rc < 0) {
        println(-2);
        return;
    }

    // Отправить HTTP-запрос
    var req = cstr("GET / HTTP/1.0\r\nHost: localhost\r\n\r\n");
    net_send(fd, req, 38);

    // Получить ответ в буфер на heap
    var buf = sys_alloc(4096);
    var received = net_recv(fd, buf, 4095);
    ptr_write_byte(buf + received, 0);   // нуль-терминатор

    println(received);    // число полученных байт

    net_close(fd);
    sys_free(buf, 4096);
}
```

### TCP-сервер (echo)

```orbitron
import "std/net";
import "std/sys";

func main() {
    var server = tcp_socket();
    net_reuseaddr(server);

    net_bind(server, INADDR_ANY, 9000);
    net_listen(server, 5);

    loop {
        var client = net_accept(server);
        if (client < 0) { break; }

        var buf = sys_alloc(1024);
        var n = net_recv(client, buf, 1023);
        if (n > 0) {
            net_send(client, buf, n);    // echo back
        }
        sys_free(buf, 1024);
        net_close(client);
    }

    net_close(server);
}
```

### Доступные константы и функции `std/net`

| Константа/функция | Описание |
|---|---|
| `AF_INET`, `AF_UNIX`, `AF_INET6` | Семейства адресов |
| `SOCK_STREAM`, `SOCK_DGRAM` | TCP / UDP |
| `INADDR_ANY`, `INADDR_LOOPBACK` | 0.0.0.0, 127.0.0.1 |
| `SO_REUSEADDR`, `SO_KEEPALIVE` | Опции сокета |
| `tcp_socket()` | Создать TCP-сокет |
| `udp_socket()` | Создать UDP-сокет |
| `net_bind(fd, ip, port)` | Привязать к адресу |
| `net_listen(fd, backlog)` | Начать прослушивание |
| `net_accept(fd)` | Принять соединение |
| `tcp_connect(fd, ip, port)` | Подключиться |
| `net_send(fd, buf, len)` | Отправить данные |
| `net_recv(fd, buf, len)` | Получить данные |
| `net_close(fd)` | Закрыть сокет |
| `net_ip(a, b, c, d)` | Собрать IPv4 из октетов |
| `net_reuseaddr(fd)` | Установить SO_REUSEADDR |
| `htons(port)`, `htonl(ip)` | Преобразование байтового порядка |

---

## 5. Базы данных — `std/db` (SQLite3)

SQLite3 позволяет хранить данные в файле или in-memory базе данных.

### Установка SQLite3

```bash
# Ubuntu / Debian
sudo apt install libsqlite3-dev

# Fedora / RHEL
sudo dnf install sqlite-devel
```

### Компиляция с SQLite3

Пока `--libs` не добавлено в CLI, скомпилируйте вручную:

```bash
orbitron myprogram.ot --emit-llvm -o out
llc out.ll -o out.s -relocation-model=pic
clang out.s -o out -lm -lsqlite3
```

Или через проект в `orbitron.toml`:
```toml
[project]
name = "myapp"
version = "0.1.0"

[build]
main = "src/main.ot"
output = "bin/myapp"
backend = "llvm"
```

### Пример использования

```orbitron
import "std/db";
import "std/sys";

func main() {
    // Открыть базу данных
    var db = db_open(cstr("data.db"));
    if (db == 0) {
        println(-1);
        return;
    }

    // Создать таблицу
    db_exec(db, cstr("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)"));

    // Вставить строку
    db_exec(db, cstr("INSERT INTO users (name, age) VALUES ('Alice', 30)"));
    var rowid = db_last_rowid(db);
    println(rowid);   // ID вставленной строки

    // Запрос с параметрами через prepare
    var stmt = db_prepare(db, cstr("SELECT id, age FROM users WHERE age > 20"));
    while (db_step(stmt) == SQLITE_ROW) {
        var id  = db_col_int(stmt, 0);
        var age = db_col_int(stmt, 1);
        println(id);
        println(age);
    }
    db_finalize(stmt);

    db_close(db);
}
```

### Привязка параметров

```orbitron
var stmt = db_prepare(db, cstr("INSERT INTO users (name, age) VALUES (?, ?)"));
sqlite3_bind_text(stmt, 1, cstr("Bob"), -1, 0);
sqlite3_bind_int64(stmt, 2, 25);
db_step(stmt);
db_finalize(stmt);
```

### Доступные функции `std/db`

| Функция | Описание |
|---|---|
| `db_open(path_addr)` | Открыть/создать БД. Возвращает handle или 0 |
| `db_exec(db, sql_addr)` | Выполнить SQL без результата |
| `db_prepare(db, sql_addr)` | Компилировать SQL. Возвращает stmt или 0 |
| `db_step(stmt)` | SQLITE_ROW / SQLITE_DONE / ошибка |
| `db_finalize(stmt)` | Освободить stmt |
| `db_col_int(stmt, col)` | Получить целочисленный столбец (0-indexed) |
| `db_col_count(stmt)` | Число столбцов в результате |
| `db_close(db)` | Закрыть БД |
| `db_last_rowid(db)` | ROWID последней вставки |
| `db_changes(db)` | Строк затронуто последней операцией |
| `SQLITE_OK`, `SQLITE_ROW`, `SQLITE_DONE` | Коды возврата |
| `SQLITE_INTEGER`, `SQLITE_TEXT`, ... | Типы столбцов |

---

## 6. Ограничения и платформа

- Все низкоуровневые возможности доступны **только в LLVM-бекенде** (`--backend llvm`).
- JVM-бекенд (`--backend jvm`) не поддерживает `&`, `*`, `cstr()`, `extern func`, `syscall`.
- Syscall-номера специфичны для **Linux x86-64**. На macOS/ARM/Windows они другие.
- Тип указателя — `i64`. На 32-битных системах использовать нельзя.
- `extern func` объявляет функции с **параметрами i64** и возвратом i64.
  Для передачи `double`/`float` используйте целочисленное представление (битовый cast).
