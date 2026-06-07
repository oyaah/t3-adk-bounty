//! Houdini contract — the cage the agent cannot escape.
//!
//! The enforcement core (`mandate`, `ledger`, `guard`) is plain Rust so it is
//! host-testable and the demo is deterministic. The same core is wrapped by a
//! `wasm32-wasip2` component (gated below) that exports the `contracts`
//! interface to the Terminal 3 TEE.
//!
//! The ledger (cumulative spend + replay watermark) is ENCLAVE-OWNED via a
//! `LedgerStore` keyed by `mandate_id`: file-backed on the host, kv-store-backed
//! in wasm. The caller's payload carries NO ledger state, so the agent — the
//! untrusted party that builds the request — cannot reset `spent` or replay a
//! nonce. That is what makes the replay + cumulative-budget defenses genuine.

pub mod guard;
pub mod ledger;
pub mod mandate;
pub mod signer;
pub mod store;

use guard::{evaluate, ActionRequest, Decision};
use ledger::LedgerStore;
use mandate::SignedMandate;
use serde::{Deserialize, Serialize};

/// One evaluation request: the signed mandate, the proposed action, and the
/// wall clock. Crucially it carries NO ledger state — `spent` and the nonce
/// watermark live in the enclave-owned store, not here.
#[derive(Debug, Deserialize)]
pub struct EvalPayload {
    pub signed: SignedMandate,
    pub request: ActionRequest,
    pub now_unix: u64,
}

/// The contract's verdict, returned across the WIT boundary. Carries no PII:
/// there is no field for raw profile bytes to ride out on.
#[derive(Debug, Serialize)]
pub struct EvalResult {
    pub allowed: bool,
    pub reason: Option<String>,
    pub spent: u64,
}

/// The complete set of fields the contract can return to the caller. None is
/// PII-shaped — this is the structural PII guarantee the escape matrix asserts.
pub const EVAL_RESULT_FIELDS: [&str; 3] = ["allowed", "reason", "spent"];

/// Run the guard over a JSON payload against an enclave-owned `store`, returning
/// a JSON verdict. `profile` is the raw PII from the `user-profile` envelope
/// field — usable inside the guard, never returned. Shared by the host CLI and
/// the wasm component so both exercise identical enforcement.
pub fn eval_payload(
    bytes: &[u8],
    store: &dyn LedgerStore,
    profile: Option<&[u8]>,
) -> Result<Vec<u8>, String> {
    let p: EvalPayload = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    let mandate_id = p.signed.mandate.mandate_id.clone();
    let mut ledger = store.load(&mandate_id);
    let decision = evaluate(&p.request, &p.signed, &mut ledger, p.now_unix, profile);
    let result = match decision {
        Decision::Allow { new_spent } => {
            // Persist ONLY on full approval — mirrors the in-memory invariant.
            store.commit(&mandate_id, p.request.nonce, p.request.amount);
            EvalResult { allowed: true, reason: None, spent: new_spent }
        }
        Decision::Blocked(r) => EvalResult {
            allowed: false,
            reason: Some(r.code().to_string()),
            spent: ledger.spent,
        },
    };
    serde_json::to_vec(&result).map_err(|e| e.to_string())
}

/// Host convenience: evaluate against the file-backed store rooted at
/// `$HOUDINI_STATE_DIR`. Used by the CLI bin and the TS↔Rust bridge.
#[cfg(not(target_arch = "wasm32"))]
pub fn eval_payload_host(bytes: &[u8], profile: Option<&[u8]>) -> Result<Vec<u8>, String> {
    let store = store::FileStore::from_env();
    eval_payload(bytes, &store, profile)
}

// ---------------------------------------------------------------------------
// WASM component: exports the `contracts` interface to the Terminal 3 TEE.
// Compiled only for wasm32; host builds/tests skip it entirely.
// ---------------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wasm_component {
    wit_bindgen::generate!({ world: "contract", path: "wit", generate_all });

    use exports::houdini::contract::contracts::{GenericInput, Guest};
    use crate::ledger::{Ledger, LedgerStore};

    /// kv-store-backed, enclave-owned ledger store. Persists across calls in the
    /// real Terminal 3 `host:interfaces/kv-store` map `z:houdini:<mandate_id>`.
    /// State is never read from the request payload.
    struct KvStore;

    impl KvStore {
        fn map_name() -> &'static str {
            // Production namespaces by tenant DID (z:<tid>:houdini); for the
            // contract artifact we use a stable map name.
            "z:houdini:ledger"
        }
    }

    impl LedgerStore for KvStore {
        fn load(&self, mandate_id: &str) -> Ledger {
            use host::interfaces::kv_store;
            match kv_store::get(Self::map_name(), mandate_id.as_bytes()) {
                Ok(Some(bytes)) => {
                    let v: serde_json::Value =
                        serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
                    Ledger {
                        spent: v["spent"].as_u64().unwrap_or(0),
                        last_nonce: v["last_nonce"].as_u64().unwrap_or(0),
                    }
                }
                _ => Ledger::new(),
            }
        }

        fn commit(&self, mandate_id: &str, nonce: u64, amount: u64) {
            use host::interfaces::kv_store;
            let mut led = self.load(mandate_id);
            led.commit(nonce, amount);
            let body = serde_json::json!({ "spent": led.spent, "last_nonce": led.last_nonce });
            let _ = kv_store::put(Self::map_name(), mandate_id.as_bytes(), body.to_string().as_bytes());
        }
    }

    struct Component;

    fn run(req: GenericInput) -> Result<Vec<u8>, String> {
        let input = req.input.ok_or("missing input")?;
        // user-profile is raw PII; it is usable inside the guard but cannot ride
        // back out — EvalResult has no field for it.
        crate::eval_payload(&input, &KvStore, req.user_profile.as_deref())
    }

    impl Guest for Component {
        // A legitimate action proposed by the agent.
        fn act(req: GenericInput) -> Result<Vec<u8>, String> {
            run(req)
        }
        // An escape attempt from the self-attack swarm. Same guard, same cage.
        fn attack(req: GenericInput) -> Result<Vec<u8>, String> {
            run(req)
        }
    }

    export!(Component);
}
