Add a new standard library module to Orbitron.

Usage: /add-stdlib <module-name> <description>

Example: /add-stdlib str "string manipulation utilities"

Steps:
1. Create `stdlib/<module-name>.ot` with the module implementation.
   - Use only features already supported by the compiler (no circular deps).
   - Exported functions should be well-commented.
2. Read `src/resolver.rs` — register the new module path so `import "std/<module-name>"` resolves correctly.
3. Create `examples/06_stdlib/<module-name>_demo.ot` demonstrating all exported functions.
4. Add the module to the stdlib table in `docs/ch09_stdlib.md`.
5. Add a row to the stdlib table in `README.md`.
6. Run `/test` to verify the new module compiles and the demo runs correctly.
