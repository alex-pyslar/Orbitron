use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::fs;

use crate::lexer::Lexer;
use crate::parser::{Parser, Stmt};

/// Recursively resolve imports starting from `entry`, collecting all AST nodes
/// into a single flat `Vec<Stmt>` in dependency order (imported module first).
///
/// `src_root` — directory used to resolve bare module paths.
///   - In project mode:    the project's `src/` directory.
///   - In single-file mode: the directory containing the entry file.
///
/// `stdlib_root` — directory of the standard library (e.g. `{exe_dir}/stdlib/`).
///   Used to resolve `std/*` imports.  `None` if stdlib is not found.
///
/// `visited` — canonicalised paths already processed (deduplication).
///
/// Import resolution rules:
///   `import "math"`      → `{src_root}/math.ot`         (project-local)
///   `import "net/http"`  → `{src_root}/net/http.ot`     (project-local)
///   `import "std/math"`  → `{stdlib_root}/math.ot`      (standard library)
pub fn resolve(
    entry:       &Path,
    src_root:    &Path,
    stdlib_root: Option<&Path>,
    visited:     &mut HashSet<PathBuf>,
) -> Result<Vec<Stmt>, String> {
    // Canonicalise to detect duplicates robustly.
    let canonical = fs::canonicalize(entry)
        .map_err(|e| format!("Не удалось найти файл '{}': {e}", entry.display()))?;

    if visited.contains(&canonical) {
        return Ok(vec![]); // already included — skip
    }
    // Check for circular imports before recursing.
    visited.insert(canonical);

    let raw = fs::read_to_string(entry)
        .map_err(|e| format!("Не удалось прочитать '{}': {e}", entry.display()))?;
    let src = raw.replace("\r\n", "\n").replace('\r', "\n");

    let tokens = Lexer::tokenize(&src)
        .map_err(|e| format!("Лексическая ошибка в '{}': {e}", entry.display()))?;
    let stmts = Parser::new(tokens).parse_program()
        .map_err(|e| format!("Синтаксическая ошибка в '{}': {e}", entry.display()))?;

    let mut result = Vec::new();
    for stmt in stmts {
        match &stmt {
            Stmt::Import { path } => {
                let import_file = if let Some(lib_name) = path.strip_prefix("std/") {
                    // Standard library import: `import "std/math"` → stdlib_root/math.ot
                    let sr = stdlib_root.ok_or_else(|| format!(
                        "Стандартная библиотека не найдена. \
                         Установите переменную ORBITRON_HOME или расположите папку 'stdlib/' \
                         рядом с исполняемым файлом orbitron.\n\
                         Импорт: \"{}\"", path
                    ))?;
                    sr.join(format!("{lib_name}.ot"))
                } else {
                    // Project-local import: src_root / path.ot
                    src_root.join(format!("{path}.ot"))
                };

                let imported = resolve(&import_file, src_root, stdlib_root, visited)
                    .map_err(|e| format!("  (импортировано из '{}')\n{e}", entry.display()))?;
                result.extend(imported);
            }
            _ => result.push(stmt),
        }
    }
    Ok(result)
}
