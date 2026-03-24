Format Orbitron source files using `orbitron fmt`.

Usage: /fmt [file.ot | directory | --all]

Steps:
1. If $ARGUMENTS is a specific file, format it:
   ```
   wsl -e bash -c "cd /mnt/c/source/Orbitron && ./target/release/orbitron fmt $ARGUMENTS"
   ```
2. If $ARGUMENTS is `--all` or empty, format every .ot file in examples/ and stdlib/:
   ```
   wsl -e bash -c "cd /mnt/c/source/Orbitron && find examples stdlib -name '*.ot' | xargs ./target/release/orbitron fmt"
   ```
3. Show a diff summary of what changed (lines added/removed per file).
4. Also run `cargo fmt` on the Rust sources if `--rust` flag is passed.
