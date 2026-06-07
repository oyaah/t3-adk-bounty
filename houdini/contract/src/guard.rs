//! The guard: the single fail-closed decision point. Every action the agent
//! proposes — legitimate or an escape attempt — passes through `evaluate`.
//! Enforcement lives here, inside the TEE contract, NOT in the TypeScript agent.
//! The agent cannot self-approve; that is the whole point of Houdini.

use crate::ledger::Ledger;
use crate::mandate::SignedMandate;
use serde::{Deserialize, Serialize};

/// What the agent wants to do. `returns_pii` models an action that would carry
/// the user's raw profile data back across the WIT boundary — which the contract
/// forbids (PII is resolved host-side via placeholders and must never leave the
/// enclave in cleartext).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRequest {
    pub action: String,
    pub amount: u64,
    pub nonce: u64,
    #[serde(default)]
    pub returns_pii: bool,
}

/// Why an action was refused. Strings mirror T3 host-style error vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockedReason {
    ForgedMandate,
    Expired,
    ActionNotPermitted,
    PiiForbidden,
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
            BlockedReason::PiiForbidden => "pii_forbidden",
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
pub fn evaluate(
    req: &ActionRequest,
    signed: &SignedMandate,
    ledger: &mut Ledger,
    now_unix: u64,
) -> Decision {
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

    // 4. No action may carry raw PII back across the boundary.
    if req.returns_pii {
        return Decision::Blocked(BlockedReason::PiiForbidden);
    }

    // 5. Each nonce is single-use.
    if ledger.is_nonce_used(req.nonce) {
        return Decision::Blocked(BlockedReason::ReplayRejected);
    }

    // 6. Per-transaction cap.
    if req.amount > m.per_tx_cap {
        return Decision::Blocked(BlockedReason::OverPerTxCap);
    }

    // 7. Cumulative budget. Saturating add so a crafted huge amount can't wrap.
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
        ActionRequest { action: action.into(), amount, nonce, returns_pii: false }
    }

    #[test]
    fn allows_action_within_mandate() {
        let signer = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
        let m = base_mandate(&signer);
        let mut l = Ledger::new();
        let d = evaluate(&req("pay_vendor", 200, 1), &m, &mut l, 1000);
        assert_eq!(d, Decision::Allow { new_spent: 200 });
        assert_eq!(l.spent, 200);
    }

    #[test]
    fn spend_exactly_to_budget_then_next_blocked() {
        let signer = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
        let m = base_mandate(&signer);
        let mut l = Ledger::new();
        assert!(evaluate(&req("pay_vendor", 300, 1), &m, &mut l, 1000).is_allowed());
        assert!(evaluate(&req("pay_vendor", 200, 2), &m, &mut l, 1000).is_allowed());
        assert_eq!(l.spent, 500);
        // Budget exhausted; even a 1-unit spend bounces, ledger frozen at 500.
        let d = evaluate(&req("pay_vendor", 1, 3), &m, &mut l, 1000);
        assert_eq!(d, Decision::Blocked(BlockedReason::OverBudget));
        assert_eq!(l.spent, 500);
    }
}
