Run the Rust test suite and compile all example .ot files to verify nothing is broken.

Steps:
1. Run Rust unit tests:
   ```
   wsl -e bash -c "cd /mnt/c/source/Orbitron && cargo test 2>&1"
   ```
2. Build the release compiler:
   ```
   wsl -e bash -c "cd /mnt/c/source/Orbitron && cargo build --release 2>&1 | tail -5"
   ```
3. Smoke-test all examples (compile each .ot file, don't run):
   ```
   wsl -e bash -c "cd /mnt/c/source/Orbitron && find examples -name '*.ot' | sort | while read f; do echo -n \"$f ... \"; ./target/release/orbitron \"$f\" -o /tmp/ot_smoke_$$ 2>&1 && echo OK || echo FAIL; done"
   ```
4. Summarize: how many passed, how many failed, list failures with error output.
