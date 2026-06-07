# Houdini — 90-second demo script

Goal: a judge understands the whole idea in under a minute and remembers it.

## One-liner to open with
> "This is Houdini — the agent that tries to escape its own limits, and can't."

## Run it (record this)

```bash
# 1. Show the cage is real: build the TEE contract to wasm and prove the export
cargo build --release --target wasm32-wasip2 --manifest-path houdini/contract/Cargo.toml
wasm-tools component wit houdini/contract/target/wasm32-wasip2/release/houdini_contract.wasm | grep export
#   -> export houdini:contract/contracts@0.1.0

# 2. The Red Room — the money line. Watch the budget gauge and the BLOCKED wall.
cargo run --bin redroom --manifest-path houdini/contract/Cargo.toml
#   -> 2 legit spends tick green, 5 escapes hit a red wall, 5/5 blocked

# 3. The proof is machine-checked, not asserted
cargo test --test escape_matrix --manifest-path houdini/contract/Cargo.toml \
           --target "$(rustc -vV | sed -n 's/host: //p')"
#   -> 6 passed: each escape blocked + ledger never moves

# 4. It's a real agent on the real SDK: TS-signed mandate, verified inside the contract
cargo build --release --bin eval --manifest-path houdini/contract/Cargo.toml
cd houdini/agent && npm test
#   -> bridge tests: TS-signed mandate ALLOWED; tampered -> forged_mandate
```

## Capture

Record the Red Room run as an asciinema cast or GIF:

```bash
asciinema rec houdini-redroom.cast \
  -c 'cargo run --bin redroom --manifest-path houdini/contract/Cargo.toml'
# or: terminalizer / vhs / a screen GIF
```

## Closing line
> "Every other agent *says* it respects its limits. Houdini *can't* exceed them — and it proves it by attacking itself."
