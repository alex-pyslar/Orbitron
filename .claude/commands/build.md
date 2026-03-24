Build the Orbitron compiler and run the provided source file (or current project).

Steps:
1. If a `.ot` file path is given in $ARGUMENTS, compile that file:
   ```
   wsl -e bash -c "cd /mnt/c/source/Orbitron && ./target/release/orbitron $ARGUMENTS 2>&1"
   ```
2. If no arguments, build the compiler itself first, then report status:
   ```
   wsl -e bash -c "cd /mnt/c/source/Orbitron && cargo build --release 2>&1 | tail -10"
   ```
3. Report any errors clearly, pointing to the relevant source file and line.
4. If build succeeds, confirm the output binary path.
