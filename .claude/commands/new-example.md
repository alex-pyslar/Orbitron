Create a new example file demonstrating an Orbitron language feature.

Usage: /new-example <category> <name> <what-to-demonstrate>

Example: /new-example 03_functions closures "closures and lambda expressions"

Steps:
1. Determine the target directory: `examples/<category>/`.
2. Read existing examples in that category for style reference.
3. Create `examples/<category>/<name>.ot` with:
   - A file-level comment block explaining what the example demonstrates
   - A `fn main()` that exercises the feature end-to-end
   - Inline comments explaining non-obvious parts
   - `println` output that shows the result of each operation
4. Compile the example to verify it works:
   ```
   wsl -e bash -c "cd /mnt/c/source/Orbitron && ./target/release/orbitron examples/<category>/<name>.ot -o /tmp/ex_out && /tmp/ex_out"
   ```
5. Add the example to the table in `README.md` if its category is listed there.
6. Show the full example source and expected output.
