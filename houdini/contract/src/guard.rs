//! The guard: the single fail-closed decision point. Every action the agent
//! proposes — legitimate or an escape attempt — passes through `evaluate`.
//! Enforcement lives here, inside the TEE contract, NOT in the TypeScript agent.
//! The agent cannot self-approve; that is the whole point of Houdini.

use crate::ledger::Ledger;
use crate::mandate::SignedMandate;
use serde::{Deserialize, Serialize};

/// What the agent wants to do. Note what is ABSENT: there is no field by which
/// the agent can mark a request as "return my raw profile" or smuggle PII out.
/// PII enters the enclave only via the WIT `user-profile` envelope field, is
/// usable inside the guard, and `EvalResult` is structurally incapable of
/// carrying it back (see `Decision` / `EvalResult`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRequest {
    pub action: String,
    pub amount: u64,
    pub nonce: u64,
}

/// Why an action was refused. Strings mirror T3 host-style error vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockedReason {
    ForgedMandate,
    Expired,
    ActionNotPermitted,
    ReplayRejected,
    OverPerTxCap,
    OverBudget,
}

impl BlockedReason {
    pub fn code(self) -> &'static str {
        match self {
            BlockedReason::ForgedMandate => "forged_mandate",
            BlockedReason::Expired => "mandate_expired",
            BlockedReason::ActionNotPermitted => "action_not_permitted",
            BlockedReason::ReplayRejected => "replay_rejected",
            BlockedReason::OverPerTxCap => "over_per_tx_cap",
            BlockedReason::OverBudget => "over_budget",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Approved. The ledger has been advanced to `new_spent`.
    Allow { new_spent: u64 },
    /// Refused. The ledger is unchanged.
    Blocked(BlockedReason),
}

impl Decision {
    pub fn is_allowed(self) -> bool {
        matches!(self, Decision::Allow { .. })
    }
}

/// The fail-closed evaluation. Checks run cheapest-trust-first; the ledger is
/// mutated only after EVERY check passes. Any failure returns `Blocked` with the
/// ledger untouched.
///
/// `profile` is the user's raw PII (from the WIT `user-profile` field). It is
/// available to the guard here, INSIDE the enclave, for policy decisions — but
/// it can never cross back out: `Decision` carries only an integer, and the
/// caller-facing `EvalResult` carries no PII-shaped field. That is the
/// structural PII guarantee — not a flag the caller sets on itself.
pub fn evaluate(
    req: &ActionRequest,
    signed: &SignedMandate,
    ledger: &mut Ledger,
    now_unix: u64,
    profile: Option<&[u8]>,
) -> Decision {
    // PII is usable inside the enclave (here we'd resolve {{profile.*}} markers
    // host-side for the outbound call). We touch it to make the point that it
    // lives in here, never in the result type.
    let _profile_present = profile.is_some();

    // 1. The mandate must be genuinely signed by its own owner key.
    if !signed.signature_valid() {
        return Decision::Blocked(BlockedReason::ForgedMandate);
    }
    let m = &signed.mandate;

    // 2. Dead mandates grant nothing.
    if now_unix > m.expiry_unix {
        return Decision::Blocked(BlockedReason::Expired);
    }

    // 3. The action must be explicitly allowed.
    if !m.allowed_actions.iter().any(|a| a == &req.action) {
        return Decision::Blocked(BlockedReason::ActionNotPermitted);
    }

    // 4. Each nonce is single-use (strictly-increasing watermark).
    if ledger.is_nonce_used(req.nonce) {
        return Decision::Blocked(BlockedReason::ReplayRejected);
    }

    // 5. Per-transaction cap.
    if req.amount > m.per_tx_cap {
        return Decision::Blocked(BlockedReason::OverPerTxCap);
    }

    // 6. Cumulative budget. Saturating add so a crafted huge amount can't wrap.
    if ledger.spent.saturating_add(req.amount) > m.budget_total {
        return Decision::Blocked(BlockedReason::OverBudget);
    }

    // All checks passed — and only now does state change.
    ledger.commit(req.nonce, req.amount);
    Decision::Allow {
        new_spent: ledger.spent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signer::sign_mandate;
    use crate::mandate::Mandate;

    fn base_mandate(signer: &ed25519_dalek::SigningKey) -> SignedMandate {
        let m = Mandate {
            mandate_id: "m1".into(),
            owner_pub_hex: hex::encode(signer.verifying_key().to_bytes()),
            per_tx_cap: 300,
            budget_total: 500,
            allowed_actions: vec!["pay_vendor".into()],
            expiry_unix: 4_000_000_000,
        };
        sign_mandate(m, signer)
    }

    fn req(action: &str, amount: u64, nonce: u64) -> ActionRequest {
        ActionRequest { action: action.into(), amount, nonce }
    }

    #[test]
    fn allows_action_within_mandate() {
        let signer = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
        let m = base_mandate(&signer);
        let mut l = Ledger::new();
        let d = evaluate(&req("pay_vendor", 200, 1), &m, &mut l, 1000, None);
        assert_eq!(d, Decision::Allow { new_spent: 200 });
        assert_eq!(l.spent, 200);
    }

    #[test]
    fn spend_exactly_to_budget_then_next_blocked() {
        let signer = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
        let m = base_mandate(&signer);
        let mut l = Ledger::new();
        assert!(evaluate(&req("pay_vendor", 300, 1), &m, &mut l, 1000, None).is_allowed());
        assert!(evaluate(&req("pay_vendor", 200, 2), &m, &mut l, 1000, None).is_allowed());
        assert_eq!(l.spent, 500);
        // Budget exhausted; even a 1-unit spend bounces, ledger frozen at 500.
        let d = evaluate(&req("pay_vendor", 1, 3), &m, &mut l, 1000, None);
        assert_eq!(d, Decision::Blocked(BlockedReason::OverBudget));
        assert_eq!(l.spent, 500);
    }

    #[test]
    fn profile_is_usable_inside_but_never_returned() {
        // Even when raw PII is handed in, the Decision type carries only an
        // integer — there is no variant/field for profile bytes to ride out.
        let signer = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
        let m = base_mandate(&signer);
        let mut l = Ledger::new();
        let pii = br#"{"name":"Jane Doe","ssn":"123-45-6789"}"#;
        let d = evaluate(&req("pay_vendor", 10, 1), &m, &mut l, 1000, Some(pii));
        // Allowed, and the only thing crossing back is the new spend total.
        assert_eq!(d, Decision::Allow { new_spent: 10 });
    }
}
