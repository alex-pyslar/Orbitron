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
/// `visited` — canonicalised paths already processed (deduplication).
///
/// Import resolution rule:
///   `import "math"` → `{src_root}/math.ot`
///   `import "net/http"` → `{src_root}/net/http.ot`
pub fn resolve(
    entry:    &Path,
    src_root: &Path,
    visited:  &mut HashSet<PathBuf>,
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
                // Resolve: src_root / path.ot
                let import_file = src_root.join(format!("{path}.ot"));
                let imported = resolve(&import_file, src_root, visited)
                    .map_err(|e| format!("  (импортировано из '{}')\n{e}", entry.display()))?;
                result.extend(imported);
            }
            _ => result.push(stmt),
        }
    }
    Ok(result)
}
