# Глава 12 — Бэкенды компиляции

Orbitron поддерживает два бэкенда компиляции, каждый с разными целями и требованиями.

---

## 12.1 — Обзор

| Бэкенд | Выходной формат | Требования | Запуск |
|--------|----------------|-----------|--------|
| **LLVM** | Нативный бинарник | `llc`, `clang`, `libm` | `./binary` |
| **JVM** | `.jar` файл | `javac`, `jar` (JDK 11+) | `java -jar binary.jar` |

По умолчанию используется бэкенд LLVM.

---

## 12.2 — Бэкенд LLVM (по умолчанию)

### Конвейер компиляции

```
source.ot
    │
    ▼  Лексер → Парсер → Резолвер импортов
    │
    ▼  Кодогенератор (3 прохода)
    │
    ▼  LLVM IR (файл .ll)
    │
    ▼  llc → Ассемблер (файл .s)
    │
    ▼  clang -lm → Нативный бинарник
```

### Три прохода кодогенерации

Генератор кода работает в три прохода, что позволяет функциям и структурам
ссылаться друг на друга в любом порядке:

| Проход | Что делает |
|--------|-----------|
| 0 | Объявляет типы структур, перечисления, константы |
| 1 | Предварительные объявления функций (forward declare) |
| 2 | Генерирует тела функций |

### Как использовать LLVM-бэкенд

```bash
# Компиляция одного файла
orbitron hello.ot                # → ./hello

# Компиляция проекта
orbitron build                   # → bin/myapp

# Флаги
orbitron build --emit-llvm       # остановиться на .ll
orbitron build --save-temps      # сохранить .ll и .s
orbitron build -v                # подробный вывод
```

### Зависимости

| Инструмент | Назначение | Установка (Ubuntu) |
|-----------|-----------|-------------------|
| `llc` | LLVM IR → ассемблер | `sudo apt install llvm-18` |
| `clang` | ассемблер → бинарник | `sudo apt install clang-18` |
| `libm` | Математика (`**` оператор) | Входит в `libc-dev` |

### Посмотреть сгенерированный LLVM IR

```bash
orbitron hello.ot --emit-llvm
cat hello.ll
```

Пример IR для простой функции:

```llvm
define i64 @add(i64 %a, i64 %b) {
entry:
  %result = add i64 %a, %b
  ret i64 %result
}
```

### Системные возможности (только LLVM)

- Указатели (`&x`, `*x`, `ptr_write`, `ptr_read`)
- Системные вызовы (`syscall`)
- Внешние C-функции (`extern func`)
- `cstr()` — C-строки
- Стандартная библиотека `std/sys`, `std/net`, `std/db`

---

## 12.3 — Бэкенд JVM

### Конвейер компиляции

```
source.ot
    │
    ▼  Лексер → Парсер → Резолвер импортов
    │
    ▼  JVM-генератор кода
    │
    ▼  Main.java (промежуточный Java-исходник)
    │
    ▼  javac → Main.class
    │
    ▼  jar cfm → output.jar
```

### Как использовать JVM-бэкенд

```bash
# Компиляция одного файла в .jar
orbitron hello.ot --backend jvm      # → hello.jar

# Компиляция проекта в .jar
orbitron build --backend jvm         # → bin/myapp.jar

# Запуск .jar
java -jar hello.jar

# Посмотреть сгенерированный Java-код
orbitron hello.ot --backend jvm --emit-java
cat hello/Main.java

# Запуск через orbitron
orbitron run --backend jvm
```

### Настройка по умолчанию в orbitron.toml

```toml
[build]
backend = "jvm"
```

### Зависимости

| Инструмент | Назначение | Установка |
|-----------|-----------|----------|
| `javac` | Компиляция Java-исходника | `sudo apt install default-jdk` |
| `jar` | Упаковка в JAR | Входит в JDK |
| `java` | Запуск JAR | Входит в JDK |

### Маппинг типов Orbitron → Java

| Orbitron | Java |
|----------|------|
| `int` | `long` |
| `float` | `double` |
| `bool` (`true`/`false`) | `long` (1L / 0L) |
| Массивы | `long[]` |
| `struct` | `static class` |
| `class` | `static class` |
| `enum` | `static final long` |

### Пример: сгенерированный Java-код

Orbitron:
```orbitron
func add(a: int, b: int): int {
    return a + b;
}

func main() {
    println(add(3, 4));
}
```

Сгенерированный `Main.java`:
```java
public class Main {
    static long add(long a, long b) {
        return a + b;
    }

    public static void main(String[] args) {
        System.out.println(add(3L, 4L));
    }
}
```

---

## 12.4 — Различия между бэкендами

| Возможность | LLVM | JVM |
|------------|------|-----|
| Нативная скорость | ✓ | Через GraalVM |
| Кросс-платформенность | Нет (нужна перекомпиляция) | ✓ (везде, где есть JVM) |
| Указатели и `&x`, `*x` | ✓ | ✗ (паника) |
| `syscall()` | ✓ | ✗ |
| `extern func` | ✓ | ✗ (игнорируется) |
| `cstr()` | ✓ | ✗ |
| `std/sys`, `std/net`, `std/db` | ✓ | ✗ |
| `std/math`, `std/bits`, `std/algo` | ✓ | ✓ |
| Структуры и классы | ✓ | ✓ |
| Перечисления | ✓ | ✓ |
| Массивы | ✓ | ✓ |
| `defer` | LLVM-специфика | `try-finally` |
| Бесконечные числа (INT_MAX) | ✓ | ✓ |

### Когда использовать LLVM

- Нужна максимальная производительность
- Программа использует системные вызовы или указатели
- Целевая платформа известна и фиксирована
- Нужны низкоуровневые возможности (сетевые операции, файлы через syscall)

### Когда использовать JVM

- Нужна кросс-платформенность (Windows, macOS, Linux одним файлом)
- Программа не использует низкоуровневые возможности
- Команда уже работает в экосистеме Java/JVM
- Нужна интеграция с Java-библиотеками (через нативные методы)

---

## 12.5 — GraalVM Native Image

Если у вас установлен GraalVM, можно скомпилировать `.jar` в нативный бинарник:

```bash
# Скомпилировать Orbitron-программу в .jar
orbitron build --backend jvm

# Скомпилировать .jar в нативный бинарник через GraalVM
native-image -jar bin/myapp.jar -o bin/myapp_native

# Запустить
./bin/myapp_native
```

Это даёт нативную скорость при сохранении JVM-семантики.

---

## 12.6 — Приоритет выбора бэкенда

| Метод | Приоритет |
|-------|-----------|
| Флаг `--backend llvm\|jvm` в CLI | Наивысший |
| Поле `[build] backend = "jvm"` в `orbitron.toml` | Средний |
| По умолчанию: `llvm` | Наименьший |

```bash
# Флаг CLI перекрывает всё остальное
orbitron build --backend jvm   # всегда JVM, игнорирует toml

# Без флага — берётся из toml или дефолтный llvm
orbitron build
```

---

## 12.7 — Отладочные флаги

```bash
# Посмотреть LLVM IR (LLVM-бэкенд)
orbitron hello.ot --emit-llvm
# → создаёт hello.ll

# Сохранить промежуточные файлы (LLVM-бэкенд)
orbitron hello.ot --save-temps
# → создаёт hello.ll и hello.s

# Посмотреть Java-исходник (JVM-бэкенд)
orbitron hello.ot --backend jvm --emit-java
# → создаёт hello/Main.java

# Подробный вывод каждого шага
orbitron hello.ot -v
```

---

← [Глава 11 — Низкоуровневое программирование](ch11_lowlevel.md) | [Справочник →](reference.md)
