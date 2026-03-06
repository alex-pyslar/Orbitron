# Low-Level Programming in Orbitron

Orbitron supports direct Linux syscall access, pointer manipulation, network programming,
and database integration through its standard library.

> **Note:** All low-level features are **LLVM backend only** (`--backend llvm`).
> The JVM backend does not support `&`, `*`, `cstr()`, `extern func`, or `syscall`.

---

## 1. Pointers

### Address-of — `&`

The `&` operator returns the address of a variable as an `int` (i64 on 64-bit platforms).

```orbitron
var x = 42;
var addr = &x;     // addr holds the address of x
println(addr);     // e.g. 140732834471960
```

Supported for variables of type `int`, `float`, and arrays (returns address of first element).

### Dereference — `*`

The `*` operator loads an `int` (i64) from an address.

```orbitron
var x = 100;
var p = &x;
var val = *p;    // val == 100
```

### Write to address — `ptr_write(addr, val)`

Built-in function that stores `val` (i64) at address `addr`.

```orbitron
var buf = [0, 0, 0];
ptr_write(&buf, 99);    // buf[0] = 99
println(*(&buf));       // 99
```

### Write a byte — `ptr_write_byte(addr, val)`

Stores one byte (low 8 bits of `val`) at address `addr`.

```orbitron
var buf = sys_alloc(64);      // allocate 64 bytes
ptr_write_byte(buf,     72);  // 'H'
ptr_write_byte(buf + 1, 105); // 'i'
ptr_write_byte(buf + 2, 0);   // null terminator
```

### Read from address — `ptr_read(addr)`

Identical to `*addr`; explicit form for readability.

```orbitron
var v = ptr_read(some_ptr);
```

### Sign extension — `sign_ext(v)`

C library functions declared via `extern func` return `int` (32-bit), but Orbitron stores
the result in an `i64` without sign extension. This means -1 from C becomes 4294967295
instead of -1 in i64.

`sign_ext(v)` truncates `v` to 32 bits and sign-extends it to 64 bits — necessary when
checking error codes (`rc < 0`).

```orbitron
extern func connect(fd: int, addr: int, len: int): int;

var rc = sign_ext(connect(fd, addr, 16));
if (rc < 0) {
    println(rc);   // -111 (ECONNREFUSED), not 4294967184
}
```

> The wrappers in `std/net` (`tcp_connect`, `net_bind`, `tcp_socket`, etc.) already
> apply `sign_ext` internally — you do not need to call it explicitly.

### C string address — `cstr("...")`

Returns the address of a null-terminated string stored as a global LLVM constant.
Use this to pass string literals to C functions.

```orbitron
var path = cstr("/tmp/data.db");
var fd = sys_open(path, O_RDONLY, 0);
```

---

## 2. Syscalls

### Built-in `syscall` function

```
syscall(nr: int, a0, a1, a2, a3, a4, a5): int
```

Executes Linux syscall number `nr` with arguments `a0`–`a5`.
Returns the kernel's return value (negative = error code).

```orbitron
// Print "Hello!\n" via SYS_WRITE directly
var msg = cstr("Hello!\n");
syscall(1, 1, msg, 7);   // write(STDOUT, msg, 7)
```

### Standard library: `std/sys`

```orbitron
import "std/sys";

func main() {
    var pid = sys_getpid();
    println(pid);

    var mem = sys_alloc(4096);     // allocate 4 KB
    ptr_write(mem, 12345);
    println(*mem);
    sys_free(mem, 4096);

    sys_exit(0);
}
```

Available constants and functions:

| Constant / Function           | Description                                   |
|-------------------------------|-----------------------------------------------|
| `STDIN`, `STDOUT`, `STDERR`   | Standard file descriptors                     |
| `SYS_READ`, `SYS_WRITE`, ...  | Syscall numbers (x86-64 Linux)                |
| `O_RDONLY`, `O_WRONLY`, `O_CREAT`, ... | open() flags                         |
| `PROT_READ`, `PROT_WRITE`, `MAP_*` | mmap() flags                           |
| `sys_write(fd, buf, len)`     | write() wrapper                               |
| `sys_read(fd, buf, len)`      | read() wrapper                                |
| `sys_open(path, flags, mode)` | open() wrapper                                |
| `sys_close(fd)`               | close() wrapper                               |
| `sys_exit(code)`              | Terminate the process                         |
| `sys_getpid()`                | PID of the current process                    |
| `sys_fork()`                  | fork()                                        |
| `sys_kill(pid, sig)`          | Send a signal                                 |
| `sys_alloc(len)`              | Allocate memory (anonymous mmap)              |
| `sys_free(addr, len)`         | Free memory (munmap)                          |
| `sys_sleep(secs)`             | Sleep for `secs` seconds                      |

---

## 3. External C Functions — `extern func`

Declare any function from libc or another C library:

```orbitron
extern func strlen(s: int): int;
extern func malloc(size: int): int;
extern func free(ptr: int): int;
extern func memcpy(dst: int, src: int, n: int): int;
extern func printf(fmt: int, ...): int;
```

After declaration, call it like any regular function:

```orbitron
var len = strlen(cstr("Hello"));
println(len);   // 5

var buf = malloc(128);
memcpy(buf, cstr("World"), 5);
printf(cstr("buf = %s\n"), buf);
free(buf);
```

> **Note:** All parameters and the return value are treated as `i64`.
> This works on 64-bit systems where sizeof(pointer) == 8.

### Variadic functions

Add `...` as the last parameter for variadic declarations:

```orbitron
extern func dprintf(fd: int, fmt: int, ...): int;

dprintf(2, cstr("Error: code %lld\n"), 42);
```

---

## 4. Networking — `std/net`

```orbitron
import "std/net";
import "std/sys";

func main() {
    // Create a TCP socket
    var fd = tcp_socket();
    if (fd < 0) {
        println(-1);
        return;
    }

    // Connect to 127.0.0.1:8080
    var ip = net_ip(127, 0, 0, 1);
    var rc = tcp_connect(fd, ip, 8080);
    if (rc < 0) {
        println(-2);
        return;
    }

    // Send an HTTP request
    var req = cstr("GET / HTTP/1.0\r\nHost: localhost\r\n\r\n");
    net_send(fd, req, 38);

    // Receive response into a heap buffer
    var buf = sys_alloc(4096);
    var received = net_recv(fd, buf, 4095);
    ptr_write_byte(buf + received, 0);   // null terminator

    println(received);    // number of bytes received

    net_close(fd);
    sys_free(buf, 4096);
}
```

### TCP Echo Server

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

### `std/net` Reference

| Constant / Function             | Description |
|---------------------------------|-------------|
| `AF_INET`, `AF_UNIX`, `AF_INET6`| Address families |
| `SOCK_STREAM`, `SOCK_DGRAM`     | TCP / UDP socket types |
| `INADDR_ANY`, `INADDR_LOOPBACK` | 0.0.0.0, 127.0.0.1 |
| `SO_REUSEADDR`, `SO_KEEPALIVE`  | Socket options |
| `tcp_socket()`                  | Create a TCP socket |
| `udp_socket()`                  | Create a UDP socket |
| `net_bind(fd, ip, port)`        | Bind to an address |
| `net_listen(fd, backlog)`       | Start listening |
| `net_accept(fd)`                | Accept a connection |
| `tcp_connect(fd, ip, port)`     | Connect to a server |
| `net_send(fd, buf, len)`        | Send data |
| `net_recv(fd, buf, len)`        | Receive data |
| `net_close(fd)`                 | Close a socket |
| `net_ip(a, b, c, d)`            | Build IPv4 from octets |
| `net_reuseaddr(fd)`             | Set SO_REUSEADDR |
| `htons(port)`, `htonl(ip)`      | Byte-order conversion |

---

## 5. Databases — `std/db` (SQLite3)

SQLite3 allows storing data in a file or an in-memory database.

### Install SQLite3

```bash
# Ubuntu / Debian
sudo apt install libsqlite3-dev

# Fedora / RHEL
sudo dnf install sqlite-devel
```

### Compiling with SQLite3

Until `--libs` support is added to the CLI, link manually:

```bash
orbitron myprogram.ot --emit-llvm -o out
llc out.ll -o out.s -relocation-model=pic
clang out.s -o out -lm -lsqlite3
```

### Example

```orbitron
import "std/db";
import "std/sys";

func main() {
    // Open / create a database
    var db = db_open(cstr("data.db"));
    if (db == 0) {
        println(-1);
        return;
    }

    // Create a table
    db_exec(db, cstr("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)"));

    // Insert a row
    db_exec(db, cstr("INSERT INTO users (name, age) VALUES ('Alice', 30)"));
    var rowid = db_last_rowid(db);
    println(rowid);   // ID of the inserted row

    // Query with a prepared statement
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

### Parameter Binding

```orbitron
var stmt = db_prepare(db, cstr("INSERT INTO users (name, age) VALUES (?, ?)"));
sqlite3_bind_text(stmt, 1, cstr("Bob"), -1, 0);
sqlite3_bind_int64(stmt, 2, 25);
db_step(stmt);
db_finalize(stmt);
```

### `std/db` Reference

| Function                  | Description |
|---------------------------|-------------|
| `db_open(path_addr)`      | Open / create a database. Returns handle or 0 |
| `db_exec(db, sql_addr)`   | Execute SQL with no result set |
| `db_prepare(db, sql_addr)`| Compile SQL. Returns stmt or 0 |
| `db_step(stmt)`           | SQLITE_ROW / SQLITE_DONE / error |
| `db_finalize(stmt)`       | Release a prepared statement |
| `db_col_int(stmt, col)`   | Get an integer column (0-indexed) |
| `db_col_count(stmt)`      | Number of columns in the result |
| `db_close(db)`            | Close the database |
| `db_last_rowid(db)`       | ROWID of the last insert |
| `db_changes(db)`          | Rows affected by the last operation |
| `SQLITE_OK`, `SQLITE_ROW`, `SQLITE_DONE` | Return codes |
| `SQLITE_INTEGER`, `SQLITE_TEXT`, ... | Column type constants |

---

## 6. Platform Notes

- All low-level features require the **LLVM backend** (`--backend llvm`).
- The JVM backend does not support `&`, `*`, `cstr()`, `extern func`, or `syscall`.
- Syscall numbers are specific to **Linux x86-64**. They differ on macOS, ARM, and Windows.
- The pointer type is `i64`. 32-bit systems are not supported.
- `extern func` declares functions with **i64 parameters** and an i64 return value.
  To pass `double`/`float`, use an integer bit-cast representation.
