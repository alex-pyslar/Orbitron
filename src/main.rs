#![allow(dead_code)] // workaround: rustc 1.93 ICE on dead_code lint near Cyrillic comments

mod error;
mod lexer;
mod parser;
mod codegen;
mod project;
mod resolver;

use crate::error::CompileError;
use crate::codegen::{CodeGen, CompileOptions};
use crate::project::load_manifest;
use inkwell::context::Context;
use std::collections::HashSet;
use std::{env, fs, path::{Path, PathBuf}, process};

const VERSION: &str = env!("CARGO_PKG_VERSION");

// ── Help text ─────────────────────────────────────────────────────────────────

fn print_help() {
    println!(
"Orbitron {ver} — компилятор языка .ot

ИСПОЛЬЗОВАНИЕ:
  orbitron new <имя>                  Создать новый проект
  orbitron build [опции]              Собрать проект (ищет orbitron.toml)
  orbitron run   [опции]              Собрать и запустить проект
  orbitron [опции] <файл.ot>          Скомпилировать один файл (обратная совместимость)

ОПЦИИ:
  -h, --help         Вывести справку и выйти
      --version      Вывести версию и выйти
  -o <файл>          Имя выходного бинарника
      --emit-llvm    Сохранить LLVM IR в <output>.ll и не компилировать дальше
      --save-temps   Сохранить промежуточные файлы (<output>.ll, <output>.s)
  -v, --verbose      Выводить шаги компиляции

ПРИМЕРЫ:
  orbitron new mycalc                 # создать проект mycalc/
  cd mycalc && orbitron build         # собрать → bin/mycalc
  cd mycalc && orbitron run           # собрать и запустить
  orbitron hello.ot                   # скомпилировать один файл

СТРУКТУРА ПРОЕКТА:
  myproject/
  ├── orbitron.toml
  └── src/
      ├── main.ot      # точка входа (содержит func main)
      └── math.ot      # модуль (import \"math\" в main.ot)

ИМПОРТ:
  import \"math\";       # загружает src/math.ot из текущего проекта

ПАЙПЛАЙН:
  .ot → Лексер → Парсер → Резолвер → AST → CodeGen → LLVM IR → llc → clang → бинарник",
        ver = VERSION
    );
}

// ── Shared build options ──────────────────────────────────────────────────────

struct BuildOpts {
    output:     Option<String>, // -o override
    emit_llvm:  bool,
    save_temps: bool,
    verbose:    bool,
}

// ── Command dispatch ──────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Err(e) = dispatch(&args) {
        eprintln!("orbitron: {}", e);
        process::exit(1);
    }
}

fn dispatch(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        print_help();
        return Ok(());
    }

    match args[1].as_str() {
        "-h" | "--help" => { print_help(); Ok(()) }
        "--version"     => { println!("orbitron {}", VERSION); Ok(()) }
        "new"           => cmd_new(args),
        "build"         => cmd_build_or_run(args, false),
        "run"           => cmd_build_or_run(args, true),
        _               => cmd_file(args),
    }
}

// ── orbitron new <name> ───────────────────────────────────────────────────────

fn cmd_new(args: &[String]) -> Result<(), String> {
    let name = args.get(2)
        .ok_or_else(|| "Использование: orbitron new <имя-проекта>".to_string())?;

    let root = PathBuf::from(name);
    if root.exists() {
        return Err(format!("Директория '{}' уже существует", name));
    }

    let src_dir = root.join("src");
    fs::create_dir_all(&src_dir)
        .map_err(|e| format!("Не удалось создать директорию: {e}"))?;

    let bin_dir = root.join("bin");
    fs::create_dir_all(&bin_dir)
        .map_err(|e| format!("Не удалось создать директорию: {e}"))?;

    // orbitron.toml
    let toml_content = format!(
r#"[project]
name = "{name}"
version = "0.1.0"

[build]
main   = "src/main.ot"
output = "bin/{name}"
"#
    );
    fs::write(root.join("orbitron.toml"), toml_content)
        .map_err(|e| format!("Не удалось создать orbitron.toml: {e}"))?;

    // src/main.ot
    let main_content = format!(
r#"func main() {{
    println({greeting});
}}
"#,
        greeting = format!("\"Привет из {}!\"", name)
    );
    fs::write(src_dir.join("main.ot"), main_content)
        .map_err(|e| format!("Не удалось создать src/main.ot: {e}"))?;

    println!("Создан проект '{}'. Попробуйте:", name);
    println!("  cd {}", name);
    println!("  orbitron run");
    Ok(())
}

// ── orbitron build / orbitron run ─────────────────────────────────────────────

fn cmd_build_or_run(args: &[String], run_after: bool) -> Result<(), String> {
    let opts = parse_build_opts(&args[2..])?;

    // Find project root: walk up from CWD looking for orbitron.toml
    let cwd = env::current_dir()
        .map_err(|e| format!("Не удалось получить рабочую директорию: {e}"))?;
    let root = find_project_root(&cwd)
        .ok_or_else(|| {
            "Файл orbitron.toml не найден.\n\
             Запустите 'orbitron new <имя>' для создания проекта, \
             или укажите файл .ot напрямую.".to_string()
        })?;

    let manifest = load_manifest(&root)?;

    let entry = root.join(&manifest.build.main);
    let src_root = entry.parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| root.join("src"));

    let raw_output = opts.output.unwrap_or_else(|| manifest.build.output.clone());
    let output_path = root.join(&raw_output);

    // Ensure bin/ dir exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Не удалось создать директорию вывода: {e}"))?;
    }

    let output_str = output_path.to_string_lossy().to_string();

    let compile_opts = CompileOptions {
        emit_llvm:  opts.emit_llvm,
        save_temps: opts.save_temps,
        verbose:    opts.verbose,
    };

    if opts.verbose {
        eprintln!("[проект] Корень: {}", root.display());
        eprintln!("[проект] Точка входа: {}", entry.display());
        eprintln!("[проект] Вывод: {}", output_str);
    }

    compile_entry(&entry, &src_root, &output_str, &compile_opts)
        .map_err(|e| e.to_string())?;

    if run_after {
        let status = process::Command::new(&output_str)
            .status()
            .map_err(|e| format!("Не удалось запустить '{}': {e}", output_str))?;
        process::exit(status.code().unwrap_or(0));
    }

    Ok(())
}

/// Walk up directory tree to find a folder containing `orbitron.toml`.
fn find_project_root(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        if dir.join("orbitron.toml").exists() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

// ── orbitron <file.ot> — single-file mode (backward compat) ──────────────────

fn cmd_file(args: &[String]) -> Result<(), String> {
    let mut input:      Option<String> = None;
    let mut output:     Option<String> = None;
    let mut emit_llvm  = false;
    let mut save_temps = false;
    let mut verbose    = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => { print_help(); process::exit(0); }
            "--version"     => { println!("orbitron {}", VERSION); process::exit(0); }
            "-o" => {
                i += 1;
                if i >= args.len() {
                    return Err("Флаг -o требует аргумент: -o <файл>".into());
                }
                output = Some(args[i].clone());
            }
            "--emit-llvm"      => emit_llvm  = true,
            "--save-temps"     => save_temps = true,
            "-v" | "--verbose" => verbose    = true,
            flag if flag.starts_with('-') => {
                return Err(format!(
                    "Неизвестный флаг '{}'\n       Используйте -h для справки", flag
                ));
            }
            _ => {
                if input.is_some() {
                    return Err(
                        "Несколько входных файлов не поддерживаются в режиме одного файла.\n\
                         Используйте 'orbitron build' для проектов.".into()
                    );
                }
                input = Some(args[i].clone());
            }
        }
        i += 1;
    }

    let input = input.ok_or_else(||
        "Не указан входной файл.\n       Используйте -h для справки".to_string()
    )?;

    let output = output.unwrap_or_else(|| {
        Path::new(&input)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("a.out")
            .to_string()
    });

    let entry = PathBuf::from(&input);
    let src_root = entry.parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let opts = CompileOptions { emit_llvm, save_temps, verbose };
    compile_entry(&entry, &src_root, &output, &opts)
        .map_err(|e| e.to_string())
}

// ── Shared compilation pipeline ───────────────────────────────────────────────

fn compile_entry(
    entry:    &Path,
    src_root: &Path,
    output:   &str,
    opts:     &CompileOptions,
) -> Result<(), CompileError> {
    if opts.verbose {
        eprintln!("[1/3] Резолвер импортов: {}", entry.display());
    }
    let mut visited = HashSet::new();
    let program = resolver::resolve(entry, src_root, &mut visited)
        .map_err(CompileError::Parse)?;

    if opts.verbose {
        eprintln!("[2/3] Генерация кода → {}", output);
    }
    let ctx = Context::create();
    let mut cg = CodeGen::new("orbitron", &ctx);
    cg.generate_program(&program);

    if opts.verbose {
        eprintln!("[3/3] Компиляция → {}", output);
    }
    cg.save_and_compile(output, opts).map_err(CompileError::Codegen)?;

    Ok(())
}

// ── Build option parser ───────────────────────────────────────────────────────

fn parse_build_opts(args: &[String]) -> Result<BuildOpts, String> {
    let mut output     = None;
    let mut emit_llvm  = false;
    let mut save_temps = false;
    let mut verbose    = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" => {
                i += 1;
                if i >= args.len() {
                    return Err("Флаг -o требует аргумент".into());
                }
                output = Some(args[i].clone());
            }
            "--emit-llvm"      => emit_llvm  = true,
            "--save-temps"     => save_temps = true,
            "-v" | "--verbose" => verbose    = true,
            flag if flag.starts_with('-') => {
                return Err(format!("Неизвестный флаг '{}'", flag));
            }
            _ => {}
        }
        i += 1;
    }

    Ok(BuildOpts { output, emit_llvm, save_temps, verbose })
}
