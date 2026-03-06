// ── Compilation pipelines: LLVM and JVM ──────────────────────────────────────

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::env;

use inkwell::context::Context;

use crate::codegen::{CodeGen, CompileOptions};
use crate::error::CompileError;
use crate::jvm::JvmOptions;
use crate::resolver;

// ── Stdlib discovery ──────────────────────────────────────────────────────────

/// Find the stdlib directory.
/// Search order:
///   1. $ORBITRON_HOME/stdlib/
///   2. {exe_dir}/stdlib/  (binary lives next to stdlib/)
pub fn find_stdlib() -> Option<PathBuf> {
    if let Ok(home) = env::var("ORBITRON_HOME") {
        let p = PathBuf::from(home).join("stdlib");
        if p.is_dir() { return Some(p); }
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            let p = dir.join("stdlib");
            if p.is_dir() { return Some(p); }
        }
    }
    None
}

// ── Project root search ───────────────────────────────────────────────────────

/// Walk up from `start` until orbitron.toml is found.
pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        if dir.join("orbitron.toml").exists() { return Some(dir); }
        if !dir.pop() { return None; }
    }
}

// ── LLVM pipeline ─────────────────────────────────────────────────────────────

pub fn compile_llvm(
    entry:    &Path,
    src_root: &Path,
    output:   &str,
    opts:     &CompileOptions,
) -> Result<(), CompileError> {
    let stdlib = find_stdlib();

    if opts.verbose {
        eprintln!("[1/3] Резолвер импортов: {}", entry.display());
        match &stdlib {
            Some(p) => eprintln!("[stdlib] {}", p.display()),
            None    => eprintln!("[stdlib] не найдена"),
        }
    }

    let mut visited = HashSet::new();
    let program = resolver::resolve(entry, src_root, stdlib.as_deref(), &mut visited)
        .map_err(CompileError::Parse)?;

    if opts.verbose { eprintln!("[2/3] Генерация LLVM IR → {}", output); }
    let ctx = Context::create();
    let mut cg = CodeGen::new("orbitron", &ctx);
    cg.generate_program(&program);

    if opts.verbose { eprintln!("[3/3] Компиляция → {}", output); }
    cg.save_and_compile(output, opts).map_err(CompileError::Codegen)
}

// ── JVM pipeline ──────────────────────────────────────────────────────────────

pub fn compile_jvm(
    entry:    &Path,
    src_root: &Path,
    output:   &str,
    opts:     &JvmOptions,
) -> Result<(), String> {
    let stdlib = find_stdlib();

    if opts.verbose {
        eprintln!("[1/2] Резолвер импортов: {}", entry.display());
        match &stdlib {
            Some(p) => eprintln!("[stdlib] {}", p.display()),
            None    => eprintln!("[stdlib] не найдена"),
        }
    }

    let mut visited = HashSet::new();
    let program = resolver::resolve(entry, src_root, stdlib.as_deref(), &mut visited)?;

    if opts.verbose { eprintln!("[2/2] JVM кодогенерация → {}.jar", output); }
    crate::jvm::generate_and_compile(&program, output, opts)
}
