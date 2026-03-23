#![allow(dead_code)] // workaround: rustc 1.93 ICE on dead_code lint near Cyrillic comments

mod cli;
mod error;
mod fmt;
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
        "fmt"           => cmd_fmt(args),
        _               => cmd_file(args),
    }
}

// ── orbitron new <name> ───────────────────────────────────────────────────────

fn cmd_new(args: &[String]) -> Result<(), String> {
    let name = args.get(2)
        .ok_or_else(|| "Usage: orbitron new <project-name>".to_string())?;

    let root = PathBuf::from(name);
    if root.exists() {
        return Err(format!("Directory '{}' already exists", name));
    }

    let src_dir = root.join("src");
    fs::create_dir_all(&src_dir)
        .map_err(|e| format!("Failed to create directory: {e}"))?;
    fs::create_dir_all(root.join("bin"))
        .map_err(|e| format!("Failed to create directory: {e}"))?;

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
        .map_err(|e| format!("Failed to create orbitron.toml: {e}"))?;

    let main_content = format!(
r#"fn main() {{
    println!("Hello from {name}!");
}}
"#
    );
    fs::write(src_dir.join("main.ot"), main_content)
        .map_err(|e| format!("Failed to create src/main.ot: {e}"))?;

    println!("Created project '{}'. Next steps:", name);
    println!("  cd {}", name);
    println!("  orbitron run");
    Ok(())
}

// ── orbitron build / orbitron run ─────────────────────────────────────────────

fn cmd_build_or_run(args: &[String], run_after: bool) -> Result<(), String> {
    let opts = parse_build_opts(&args[2..])?;

    let cwd = env::current_dir()
        .map_err(|e| format!("Cannot get current directory: {e}"))?;
    let root = find_project_root(&cwd)
        .ok_or_else(|| {
            "orbitron.toml not found.\n\
             Run 'orbitron new <name>' to create a project, \
             or pass a .ot file directly.".to_string()
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
            .map_err(|e| format!("Cannot create output directory: {e}"))?;
    }
    let output_str = output_path.to_string_lossy().to_string();

    if opts.verbose {
        eprintln!("[project] Root:    {}", root.display());
        eprintln!("[project] Entry:   {}", entry.display());
        eprintln!("[project] Backend: {}", backend.name());
        eprintln!("[project] Output:  {}", output_str);
    }

    match &backend {
        Backend::Llvm => {
            let co = CompileOptions { emit_llvm: opts.emit_llvm, save_temps: opts.save_temps, verbose: opts.verbose };
            compile_llvm(&entry, &src_root, &output_str, &co)
                .map_err(|e| e.to_string())?;
            if run_after {
                let status = process::Command::new(&output_str)
                    .status()
                    .map_err(|e| format!("Failed to run '{}': {e}", output_str))?;
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
                    .map_err(|e| format!("java not found: {e}"))?;
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
                if i >= args.len() { return Err("-o flag requires an argument".into()); }
                output = Some(args[i].clone());
            }
            "--backend" => {
                i += 1;
                if i >= args.len() { return Err("--backend requires an argument: llvm | jvm".into()); }
                backend = Some(Backend::from_str(&args[i])
                    .ok_or_else(|| format!("Unknown backend '{}'. Use llvm or jvm", args[i]))?);
            }
            "--emit-llvm"      => emit_llvm  = true,
            "--emit-java"      => emit_java   = true,
            "--save-temps"     => save_temps  = true,
            "-v" | "--verbose" => verbose     = true,
            flag if flag.starts_with('-') => {
                return Err(format!("Unknown flag '{}'\n       Use -h for help", flag));
            }
            _ => {
                if input.is_some() {
                    return Err(
                        "Multiple input files are not supported.\n\
                         Use 'orbitron build' for multi-file projects.".into()
                    );
                }
                input = Some(args[i].clone());
            }
        }
        i += 1;
    }

    let input = input.ok_or_else(||
        "No input file specified.\n       Use -h for help".to_string()
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

// ── orbitron fmt [--write] [file.ot ...] ──────────────────────────────────────

fn cmd_fmt(args: &[String]) -> Result<(), String> {
    let mut files: Vec<String> = Vec::new();
    let mut write_back = false;

    for arg in args.iter().skip(2) {
        match arg.as_str() {
            "--write" | "-w" => write_back = true,
            flag if flag.starts_with('-') =>
                return Err(format!("Unknown fmt flag '{}'.\nUsage: orbitron fmt [--write] [file.ot ...]", flag)),
            _ => files.push(arg.clone()),
        }
    }

    // If no files given, find all .ot files in the current project src/
    if files.is_empty() {
        let cwd = std::env::current_dir()
            .map_err(|e| format!("Cannot get current directory: {e}"))?;
        let src_dir = cwd.join("src");
        let scan_dir = if src_dir.exists() { src_dir } else { cwd };
        for entry in walkdir_ot(&scan_dir)? {
            files.push(entry.to_string_lossy().to_string());
        }
        if files.is_empty() {
            eprintln!("No .ot files found.");
            return Ok(());
        }
    }

    let mut any_changed = false;
    for file in &files {
        let src = fs::read_to_string(file)
            .map_err(|e| format!("Cannot read '{}': {e}", file))?;
        let formatted = fmt::format_source(&src)
            .map_err(|e| format!("{}: {}", file, e))?;

        if write_back {
            if formatted != src {
                fs::write(file, &formatted)
                    .map_err(|e| format!("Cannot write '{}': {e}", file))?;
                println!("fmt: {}", file);
                any_changed = true;
            }
        } else {
            print!("{}", formatted);
        }
    }

    if write_back && !any_changed {
        println!("All files are already formatted.");
    }
    Ok(())
}

/// Recursively collect .ot file paths under a directory.
fn walkdir_ot(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Cannot read directory '{}': {e}", dir.display()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walkdir_ot(&path)?);
        } else if path.extension().and_then(|e| e.to_str()) == Some("ot") {
            out.push(path);
        }
    }
    Ok(out)
}
