Compile and immediately run an Orbitron source file.

Usage: /run <file.ot> [args...]

Steps:
1. Take the file path from $ARGUMENTS (e.g. `examples/01_basics/hello.ot`).
2. Compile and run via WSL:
   ```
   wsl -e bash -c "cd /mnt/c/source/Orbitron && ./target/release/orbitron $ARGUMENTS && ./$(basename $ARGUMENTS .ot)"
   ```
3. Show stdout/stderr output.
4. If the binary requires stdin input, note that in the response.
5. Clean up the produced binary afterwards unless --save-temps was passed.
