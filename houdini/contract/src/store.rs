//! Host-side persistent `LedgerStore` — a file-backed JSON store keyed by
//! `mandate_id`. Used by the CLI (`eval` bin), the Red Room, and the TS↔Rust
//! bridge so cumulative budget and replay state survive ACROSS calls inside the
//! evaluator, exactly as the enclave's kv-store does in production.
//!
//! The state directory is `$HOUDINI_STATE_DIR` if set, else a stable per-user
//! temp dir. Tests/CLI get a fresh, independent scenario by pointing
//! `HOUDINI_STATE_DIR` at a new directory — NOT by editing the request payload.

#[cfg(not(target_arch = "wasm32"))]
mod host_impl {
    use crate::ledger::{Ledger, LedgerStore};
    use std::path::PathBuf;

    pub struct FileStore {
        dir: PathBuf,
    }

    impl FileStore {
        /// Open the store rooted at `$HOUDINI_STATE_DIR` (or a temp fallback).
        pub fn from_env() -> Self {
            let dir = std::env::var_os("HOUDINI_STATE_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::temp_dir().join("houdini-ledger"));
            let _ = std::fs::create_dir_all(&dir);
            Self { dir }
        }

        fn path(&self, mandate_id: &str) -> PathBuf {
            // mandate_id is owner-signed and verified before any store access in
            // the guard path; sanitize anyway so it is always a single filename.
            let safe: String = mandate_id
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
                .collect();
            self.dir.join(format!("{safe}.json"))
        }
    }

    impl LedgerStore for FileStore {
        fn load(&self, mandate_id: &str) -> Ledger {
            // EAFP: a missing/garbled file means a fresh ledger.
            let Ok(bytes) = std::fs::read(self.path(mandate_id)) else {
                return Ledger::new();
            };
            let v: serde_json::Value = match serde_json::from_slice(&bytes) {
                Ok(v) => v,
                Err(_) => return Ledger::new(),
            };
            Ledger {
                spent: v["spent"].as_u64().unwrap_or(0),
                last_nonce: v["last_nonce"].as_u64().unwrap_or(0),
            }
        }

        fn commit(&self, mandate_id: &str, nonce: u64, amount: u64) {
            let mut led = self.load(mandate_id);
            led.commit(nonce, amount);
            let body = serde_json::json!({ "spent": led.spent, "last_nonce": led.last_nonce });
            let _ = std::fs::write(self.path(mandate_id), body.to_string());
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use host_impl::FileStore;
