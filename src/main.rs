#![allow(dead_code)] // workaround: rustc 1.93 ICE on dead_code lint near Cyrillic comments

mod cli;
mod error;
mod jvm;
mod lexer;
mod parser;
mod codegen;
mod pipeline;
mod project;
mod resolver;

use cli::{Backend, print_help, parse_build_opts};
use crate::codegen::CompileOptions;
use crate::jvm::JvmOptions;
use crate::project::load_manifest;
use crate::pipeline::{compile_llvm, compile_jvm, find_project_root};
use std::{env, fs, path::{Path, PathBuf}, process};

const VERSION: &str = env!("CARGO_PKG_VERSION");

// ── Entry point ───────────────────────────────────────────────────────────────

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
    fs::create_dir_all(root.join("bin"))
        .map_err(|e| format!("Не удалось создать директорию: {e}"))?;

    let toml_content = format!(
r#"[project]
name = "{name}"
version = "0.1.0"

[build]
main    = "src/main.ot"
output  = "bin/{name}"
backend = "llvm"
"#
    );
    fs::write(root.join("orbitron.toml"), toml_content)
        .map_err(|e| format!("Не удалось создать orbitron.toml: {e}"))?;

    let main_content = format!(
r#"func main() {{
    println("Привет из {name}!");
}}
"#
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

    let cwd = env::current_dir()
        .map_err(|e| format!("Не удалось получить рабочую директорию: {e}"))?;
    let root = find_project_root(&cwd)
        .ok_or_else(|| {
            "Файл orbitron.toml не найден.\n\
             Запустите 'orbitron new <имя>' для создания проекта, \
             или укажите файл .ot напрямую.".to_string()
        })?;

    let manifest = load_manifest(&root)?;

    // CLI flag > toml value > default (llvm)
    let backend = opts.backend.clone().unwrap_or_else(|| {
        Backend::from_str(&manifest.build.backend).unwrap_or(Backend::Llvm)
    });

    let entry = root.join(&manifest.build.main);
    let src_root = entry.parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| root.join("src"));

    let raw_output = opts.output.unwrap_or_else(|| manifest.build.output.clone());
    let output_path = root.join(&raw_output);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Не удалось создать директорию вывода: {e}"))?;
    }
    let output_str = output_path.to_string_lossy().to_string();

    if opts.verbose {
        eprintln!("[проект] Корень: {}", root.display());
        eprintln!("[проект] Точка входа: {}", entry.display());
        eprintln!("[проект] Бэкенд: {}", backend.name());
        eprintln!("[проект] Вывод: {}", output_str);
    }

    match &backend {
        Backend::Llvm => {
            let co = CompileOptions { emit_llvm: opts.emit_llvm, save_temps: opts.save_temps, verbose: opts.verbose };
            compile_llvm(&entry, &src_root, &output_str, &co)
                .map_err(|e| e.to_string())?;
            if run_after {
                let status = process::Command::new(&output_str)
                    .status()
                    .map_err(|e| format!("Не удалось запустить '{}': {e}", output_str))?;
                process::exit(status.code().unwrap_or(0));
            }
        }
        Backend::Jvm => {
            let jo = JvmOptions { emit_java: opts.emit_java, verbose: opts.verbose };
            compile_jvm(&entry, &src_root, &output_str, &jo)?;
            if run_after {
                let jar = format!("{}.jar", output_str);
                let status = process::Command::new("java")
                    .args(["-jar", &jar])
                    .status()
                    .map_err(|e| format!("java не найден: {e}"))?;
                process::exit(status.code().unwrap_or(0));
            }
        }
    }
    Ok(())
}

// ── orbitron <file.ot> — single-file mode ────────────────────────────────────

fn cmd_file(args: &[String]) -> Result<(), String> {
    let mut input:      Option<String> = None;
    let mut output:     Option<String> = None;
    let mut backend:    Option<Backend> = None;
    let mut emit_llvm  = false;
    let mut emit_java  = false;
    let mut save_temps = false;
    let mut verbose    = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => { print_help(); process::exit(0); }
            "--version"     => { println!("orbitron {}", VERSION); process::exit(0); }
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
                return Err(format!("Неизвестный флаг '{}'\n       Используйте -h для справки", flag));
            }
            _ => {
                if input.is_some() {
                    return Err(
                        "Несколько входных файлов не поддерживаются.\n\
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

    let entry    = PathBuf::from(&input);
    let src_root = entry.parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    match backend.unwrap_or(Backend::Llvm) {
        Backend::Llvm => {
            let co = CompileOptions { emit_llvm, save_temps, verbose };
            compile_llvm(&entry, &src_root, &output, &co)
                .map_err(|e| e.to_string())
        }
        Backend::Jvm => {
            let jo = JvmOptions { emit_java, verbose };
            compile_jvm(&entry, &src_root, &output, &jo)
        }
    }
}
