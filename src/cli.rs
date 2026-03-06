// ── CLI: Backend enum, BuildOpts, help text, arg parser ──────────────────────

const VERSION: &str = env!("CARGO_PKG_VERSION");

// ── Backend ───────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub enum Backend { Llvm, Jvm }

impl Backend {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "llvm" => Some(Backend::Llvm),
            "jvm"  => Some(Backend::Jvm),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self { Backend::Llvm => "llvm", Backend::Jvm => "jvm" }
    }
}

// ── Build options ─────────────────────────────────────────────────────────────

pub struct BuildOpts {
    pub output:     Option<String>,
    /// CLI override; None → read from manifest / default (llvm)
    pub backend:    Option<Backend>,
    pub emit_llvm:  bool,
    pub emit_java:  bool,
    pub save_temps: bool,
    pub verbose:    bool,
}

// ── Help text ─────────────────────────────────────────────────────────────────

pub fn print_help() {
    println!(
"Orbitron {ver} — компилятор языка .ot

ИСПОЛЬЗОВАНИЕ:
  orbitron new <имя>                  Создать новый проект
  orbitron build [опции]              Собрать проект (ищет orbitron.toml)
  orbitron run   [опции]              Собрать и запустить проект
  orbitron [опции] <файл.ot>          Скомпилировать один файл

ОПЦИИ:
  -h, --help              Вывести справку и выйти
      --version           Вывести версию и выйти
  -o <файл>               Имя выходного файла
      --backend llvm|jvm  Бэкенд компиляции (по умолчанию: llvm)
      --emit-llvm         Сохранить LLVM IR и не компилировать дальше (llvm)
      --emit-java         Сохранить Java источник и не компилировать дальше (jvm)
      --save-temps        Сохранить промежуточные файлы
  -v, --verbose           Выводить шаги компиляции

БЭКЕНДЫ:
  llvm   → нативный бинарник (через llc + clang)
  jvm    → байткод JVM (через javac + jar, запуск: java -jar ...)
           Работает на стандартной JVM и GraalVM JDK.
           GraalVM нативный образ: native-image -jar <файл>.jar

КОНФИГУРАЦИЯ ПРОЕКТА (orbitron.toml):
  [project]
  name = \"myapp\"
  version = \"0.1.0\"

  [build]
  main    = \"src/main.ot\"
  output  = \"bin/myapp\"
  backend = \"llvm\"    # или \"jvm\"

СТАНДАРТНАЯ БИБЛИОТЕКА:
  import \"std/math\";   # математические функции (abs, max, min, gcd, ...)
  import \"std/bits\";   # битовые операции (bit_count, is_pow2, ...)
  import \"std/algo\";   # вспомогательные алгоритмы (min3, max3, lerp, ...)

  Папка stdlib/ должна лежать рядом с бинарником orbitron,
  либо установите переменную окружения ORBITRON_HOME.

ПРИМЕРЫ:
  orbitron new mycalc                     # создать проект
  cd mycalc && orbitron build             # собрать (llvm)
  cd mycalc && orbitron run               # запустить
  orbitron hello.ot                       # один файл (llvm)
  orbitron hello.ot --backend jvm         # один файл (jvm)
  orbitron build --backend jvm            # проект (jvm)
  orbitron build --emit-llvm              # только LLVM IR
  orbitron build -v                       # подробный вывод

СТРУКТУРА ПРОЕКТА:
  myproject/
  ├── orbitron.toml
  └── src/
      ├── main.ot
      └── utils.ot

ПАЙПЛАЙН (llvm):
  .ot → Лексер → Парсер → Резолвер → AST → CodeGen → LLVM IR → llc → clang → бинарник

ПАЙПЛАЙН (jvm):
  .ot → Лексер → Парсер → Резолвер → AST → JvmCodeGen → Main.java → javac → .jar",
        ver = VERSION
    );
}

// ── Argument parser ───────────────────────────────────────────────────────────

pub fn parse_build_opts(args: &[String]) -> Result<BuildOpts, String> {
    let mut output     = None;
    let mut backend    = None;
    let mut emit_llvm  = false;
    let mut emit_java  = false;
    let mut save_temps = false;
    let mut verbose    = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" => {
                i += 1;
                if i >= args.len() { return Err("Флаг -o требует аргумент".into()); }
                output = Some(args[i].clone());
            }
            "--backend" => {
                i += 1;
                if i >= args.len() { return Err("--backend требует аргумент: llvm | jvm".into()); }
                backend = Some(Backend::from_str(&args[i])
                    .ok_or_else(|| format!("Неизвестный бэкенд '{}'. Используйте llvm или jvm", args[i]))?);
            }
            "--emit-llvm"      => emit_llvm  = true,
            "--emit-java"      => emit_java   = true,
            "--save-temps"     => save_temps  = true,
            "-v" | "--verbose" => verbose     = true,
            flag if flag.starts_with('-') => {
                return Err(format!("Неизвестный флаг '{}'. Используйте -h для справки", flag));
            }
            _ => {}
        }
        i += 1;
    }

    Ok(BuildOpts { output, backend, emit_llvm, emit_java, save_temps, verbose })
}
