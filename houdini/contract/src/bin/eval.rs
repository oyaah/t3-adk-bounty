//! Headless evaluator: reads an EvalPayload JSON on stdin, runs the real guard
//! against the enclave-owned (file-backed) ledger store, writes an EvalResult
//! JSON to stdout. This is the bridge the TypeScript agent drives — proving the
//! agent talks to the genuine contract logic, not a mock.
//!
//! The ledger persists under `$HOUDINI_STATE_DIR`; the payload carries no ledger
//! state, so a caller cannot reset `spent` or replay a nonce. Tests/CLI get a
//! fresh scenario by pointing `HOUDINI_STATE_DIR` at a new dir.
//!
//! Run: echo '<json>' | cargo run --bin eval

// Host-only tool — the wasm component is the lib, not this bin. The cfg-gated
// stub keeps `cargo build --target wasm32-wasip2` (which globs all bins) green.
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use std::io::{Read, Write};
    let mut input = Vec::new();
    if std::io::stdin().read_to_end(&mut input).is_err() {
        eprintln!("eval: failed to read stdin");
        std::process::exit(2);
    }
    match houdini_contract::eval_payload_host(&input, None) {
        Ok(out) => {
            std::io::stdout().write_all(&out).ok();
            println!();
        }
        Err(e) => {
            eprintln!("eval: {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {}
