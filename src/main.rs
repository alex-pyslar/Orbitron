#![allow(dead_code)] // workaround: rustc 1.93 ICE on dead_code lint near Cyrillic comments

mod error;
mod lexer;
mod parser;
mod codegen;

use crate::error::CompileError;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::codegen::{CodeGen, CompileOptions};
use inkwell::context::Context;
use std::{env, fs, path::Path, process};

const VERSION: &str = env!("CARGO_PKG_VERSION");

// ── Help text ─────────────────────────────────────────────────────────────────

fn print_help() {
    println!(
"Orbitron {ver} — компилятор языка .ot

ИСПОЛЬЗОВАНИЕ:
  orbitron [опции] <файл.ot>

ОПЦИИ:
  -h, --help         Вывести справку и выйти
      --version      Вывести версию и выйти
  -o <файл>          Имя выходного бинарника
                     (по умолчанию: имя исходника без расширения .ot)
      --emit-llvm    Сохранить LLVM IR в <output>.ll и не компилировать дальше
      --save-temps   Сохранить промежуточные файлы (<output>.ll, <output>.s)
  -v, --verbose      Выводить шаги компиляции

ПРИМЕРЫ:
  orbitron hello.ot                      # компиляция → ./hello
  orbitron -o myapp hello.ot             # компиляция → ./myapp
  orbitron --emit-llvm hello.ot          # только LLVM IR → hello.ll
  orbitron --save-temps -o out hello.ot  # сохранить out.ll и out.s
  orbitron -v examples/fibonacci.ot      # подробный вывод шагов компиляции

ПАЙПЛАЙН:
  .ot → Лексер → Парсер → AST → CodeGen → LLVM IR → llc → .s → clang → бинарник",
        ver = VERSION
    );
}

// ── CLI config ────────────────────────────────────────────────────────────────

struct Config {
    input:      String,
    output:     String,
    emit_llvm:  bool,
    save_temps: bool,
    verbose:    bool,
}

fn parse_args(args: &[String]) -> Result<Config, String> {
    let mut input:      Option<String> = None;
    let mut output:     Option<String> = None;
    let mut emit_llvm  = false;
    let mut save_temps = false;
    let mut verbose    = false;

    let mut i = 1; // skip binary name
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                process::exit(0);
            }
            "--version" => {
                println!("orbitron {}", VERSION);
                process::exit(0);
            }
            "-o" => {
                i += 1;
                if i >= args.len() {
                    return Err("флаг -o требует аргумент: -o <файл>".into());
                }
                output = Some(args[i].clone());
            }
            "--emit-llvm"      => emit_llvm  = true,
            "--save-temps"     => save_temps = true,
            "-v" | "--verbose" => verbose    = true,
            flag if flag.starts_with('-') => {
                return Err(format!(
                    "неизвестный флаг '{}'\n       Используйте -h для справки",
                    flag
                ));
            }
            _ => {
                if input.is_some() {
                    return Err(
                        "несколько входных файлов не поддерживаются.\n       Используйте -h для справки".into()
                    );
                }
                input = Some(args[i].clone());
            }
        }
        i += 1;
    }

    let input = input.ok_or_else(||
        "не указан входной файл.\n       Используйте -h для справки".to_string()
    )?;

    // Default output: strip .ot extension, fallback to "a.out"
    let output = output.unwrap_or_else(|| {
        Path::new(&input)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("a.out")
            .to_string()
    });

    Ok(Config { input, output, emit_llvm, save_temps, verbose })
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = match parse_args(&args) {
        Ok(c)  => c,
        Err(e) => {
            eprintln!("orbitron: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = run(config) {
        eprintln!("orbitron: {}", e);
        process::exit(1);
    }
}

fn run(config: Config) -> Result<(), CompileError> {
    let src = fs::read_to_string(&config.input)
        .map_err(|e| CompileError::Io(format!("не удалось прочитать '{}': {}", config.input, e)))?
        .replace("\r\n", "\n")
        .replace('\r', "\n");

    if config.verbose {
        eprintln!("[1/3] Лексический анализ:    {}", config.input);
    }
    let tokens = Lexer::tokenize(&src).map_err(CompileError::Lex)?;

    if config.verbose {
        eprintln!("[2/3] Синтаксический анализ");
    }
    let program = Parser::new(tokens).parse_program().map_err(CompileError::Parse)?;

    if config.verbose {
        eprintln!("[3/3] Генерация кода → {}", config.output);
    }
    let ctx = Context::create();
    let mut cg = CodeGen::new("orbitron", &ctx);
    cg.generate_program(&program);

    let opts = CompileOptions {
        emit_llvm:  config.emit_llvm,
        save_temps: config.save_temps,
        verbose:    config.verbose,
    };
    cg.save_and_compile(&config.output, &opts).map_err(CompileError::Codegen)?;

    Ok(())
}
