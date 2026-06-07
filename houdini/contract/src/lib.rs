//! Houdini contract — the cage the agent cannot escape.
//!
//! The enforcement core (`mandate`, `ledger`, `guard`) is plain Rust so it is
//! host-testable and the demo is deterministic. The same core is wrapped by a
//! `wasm32-wasip2` component (gated below) that exports the `contracts`
//! interface to the Terminal 3 TEE. Production persists the ledger in
//! `host:interfaces/kv-store@2.1.0`; the offline demo passes ledger state in the
//! call payload (see KTD-2: local enclave-faithful harness).

pub mod guard;
pub mod ledger;
pub mod mandate;
pub mod signer;

use guard::{evaluate, ActionRequest, Decision};
use ledger::Ledger;
use mandate::SignedMandate;
use serde::{Deserialize, Serialize};

/// One stateless evaluation request: the signed mandate, the proposed action,
/// and the current ledger state the host supplies from KV.
#[derive(Debug, Deserialize)]
pub struct EvalPayload {
    pub signed: SignedMandate,
    pub request: ActionRequest,
    #[serde(default)]
    pub prior_spent: u64,
    #[serde(default)]
    pub used_nonces: Vec<u64>,
    pub now_unix: u64,
}

/// The contract's verdict, returned across the WIT boundary. Carries no PII.
#[derive(Debug, Serialize)]
pub struct EvalResult {
    pub allowed: bool,
    pub reason: Option<String>,
    pub spent: u64,
}

/// Run the guard over a JSON payload and return a JSON verdict. Shared by the
/// host demo and the wasm component so both exercise identical enforcement.
pub fn eval_payload(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let p: EvalPayload = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    let mut ledger = Ledger {
        spent: p.prior_spent,
        used_nonces: p.used_nonces.into_iter().collect(),
    };
    let decision = evaluate(&p.request, &p.signed, &mut ledger, p.now_unix);
    let result = match decision {
        Decision::Allow { new_spent } => EvalResult {
            allowed: true,
            reason: None,
            spent: new_spent,
        },
        Decision::Blocked(r) => EvalResult {
            allowed: false,
            reason: Some(r.code().to_string()),
            spent: ledger.spent,
        },
    };
    serde_json::to_vec(&result).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// WASM component: exports the `contracts` interface to the Terminal 3 TEE.
// Compiled only for wasm32; host builds/tests skip it entirely.
// ---------------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wasm_component {
    wit_bindgen::generate!({ world: "contract", path: "wit" });

    use exports::houdini::contract::contracts::{GenericInput, Guest};

    struct Component;

    fn run(req: GenericInput) -> Result<Vec<u8>, String> {
        let input = req.input.ok_or("missing input")?;
        crate::eval_payload(&input)
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
