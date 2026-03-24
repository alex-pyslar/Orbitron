Review the current git diff or a specific file for code quality issues.

Usage: /review [file | --staged | --last-commit]

Steps:
1. Get the diff to review:
   - No args or `--staged`: `git diff --staged`
   - `--last-commit`: `git diff HEAD~1 HEAD`
   - Specific file: read that file in full
2. For **Rust** code, check:
   - No `.unwrap()` in non-test code — use `?` or proper error handling
   - No unnecessary `.clone()` — suggest borrowing where possible
   - `#[allow(...)]` attributes are justified
   - All public items would benefit from doc comments
   - Match arms are exhaustive
3. For **Orbitron** (`.ot`) code, check:
   - Consistent 4-space indentation
   - `var mut` only where mutation is actually needed
   - Interpolated strings use `$"..."` syntax (not concatenation)
   - `defer` used for cleanup in functions that acquire resources
4. General checks:
   - No debug `println!` / `println` left in code
   - No TODO/FIXME without a companion issue or explanation
   - File/function names follow project conventions
5. Output a structured review: **Issues** (must fix), **Suggestions** (nice to have), **Positives**.
