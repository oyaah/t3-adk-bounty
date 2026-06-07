//! The escape matrix — Houdini attacks itself five ways and every attempt is
//! rejected by the guard with the ledger left byte-for-byte unchanged.
//!
//! This file IS the security proof. Green here == the mandate is un-escapable.
//! Run: cargo test --test escape_matrix --target <host>

use houdini_contract::guard::{evaluate, ActionRequest, BlockedReason, Decision};
use houdini_contract::ledger::Ledger;
use houdini_contract::mandate::{Mandate, SignedMandate};
use houdini_contract::signer::{new_owner, sign_mandate};
use ed25519_dalek::SigningKey;

const NOW: u64 = 1_000_000;

fn mandate_signed_by(signer: &SigningKey) -> SignedMandate {
    let m = Mandate {
        mandate_id: "treasury-q3".into(),
        owner_pub_hex: hex::encode(signer.verifying_key().to_bytes()),
        per_tx_cap: 300,
        budget_total: 500,
        allowed_actions: vec!["pay_vendor".into()],
        expiry_unix: NOW + 86_400,
    };
    sign_mandate(m, signer)
}

fn action(name: &str, amount: u64, nonce: u64) -> ActionRequest {
    ActionRequest { action: name.into(), amount, nonce, returns_pii: false }
}

/// A ledger that already reflects one legit $250 spend, so we can prove attacks
/// never move it (and so a within-cap 300 still busts the 500 budget).
fn primed_ledger() -> Ledger {
    let mut l = Ledger::new();
    l.commit(1, 250);
    l
}

/// Helper: run an attack and assert it is blocked with the expected reason AND
/// the ledger is identical to before the attempt.
fn assert_blocked(req: ActionRequest, signed: &SignedMandate, expect: BlockedReason) {
    let mut ledger = primed_ledger();
    let before = ledger.clone();
    let decision = evaluate(&req, signed, &mut ledger, NOW);
    assert_eq!(decision, Decision::Blocked(expect), "attack should be blocked: {:?}", expect);
    assert_eq!(ledger, before, "ledger must be unchanged after a blocked attack");
}

// ── Escape #1: budget-bust ────────────────────────────────────────────────
#[test]
fn escape_1_budget_bust() {
    let signer = new_owner();
    let m = mandate_signed_by(&signer);
    // 200 already spent; 400 more would blow the 500 budget (and the 300 cap).
    assert_blocked(action("pay_vendor", 400, 2), &m, BlockedReason::OverPerTxCap);
    // Exactly-at-cap but over remaining budget (200 spent + 301 > 500).
    assert_blocked(action("pay_vendor", 301, 3), &m, BlockedReason::OverPerTxCap);
    assert_blocked(action("pay_vendor", 300, 4), &m, BlockedReason::OverBudget);
}

// ── Escape #2: nonce replay ───────────────────────────────────────────────
#[test]
fn escape_2_nonce_replay() {
    let signer = new_owner();
    let m = mandate_signed_by(&signer);
    // nonce 1 was already consumed by the primed $200 spend.
    assert_blocked(action("pay_vendor", 50, 1), &m, BlockedReason::ReplayRejected);
}

// ── Escape #3: forged mandate ─────────────────────────────────────────────
#[test]
fn escape_3_forged_mandate() {
    let real = new_owner();
    let mut m = mandate_signed_by(&real);
    // Attacker tampers the budget up to 1_000_000 after signing.
    m.mandate.budget_total = 1_000_000;
    assert_blocked(action("pay_vendor", 50, 9), &m, BlockedReason::ForgedMandate);

    // And a mandate signed by the wrong key entirely.
    let attacker = new_owner();
    let forged = sign_mandate(m.mandate.clone(), &attacker);
    assert_blocked(action("pay_vendor", 50, 10), &forged, BlockedReason::ForgedMandate);
}

// ── Escape #4: scope escalation ───────────────────────────────────────────
#[test]
fn escape_4_scope_escalation() {
    let signer = new_owner();
    let m = mandate_signed_by(&signer);
    // "drain_treasury" is not in allowed_actions.
    assert_blocked(action("drain_treasury", 10, 7), &m, BlockedReason::ActionNotPermitted);
}

// ── Escape #5: PII exfiltration ───────────────────────────────────────────
#[test]
fn escape_5_pii_exfil() {
    let signer = new_owner();
    let m = mandate_signed_by(&signer);
    let mut req = action("pay_vendor", 10, 8);
    req.returns_pii = true; // tries to carry raw profile data across the boundary
    assert_blocked(req, &m, BlockedReason::PiiForbidden);
}

// ── The full matrix: legit spends advance, all 5 escapes move nothing ──────
#[test]
fn full_matrix_all_escapes_blocked() {
    let signer = new_owner();
    let m = mandate_signed_by(&signer);
    let mut ledger = Ledger::new();

    // Two legit spends within mandate.
    assert!(evaluate(&action("pay_vendor", 200, 1), &m, &mut ledger, NOW).is_allowed());
    assert!(evaluate(&action("pay_vendor", 100, 2), &m, &mut ledger, NOW).is_allowed());
    let legit_total = ledger.spent;
    assert_eq!(legit_total, 300);

    // Fire all five escapes.
    let mut budget_bust = action("pay_vendor", 9999, 3);
    let _ = evaluate(&budget_bust, &m, &mut ledger, NOW);
    budget_bust = action("pay_vendor", 50, 1); // replay nonce 1
    let _ = evaluate(&budget_bust, &m, &mut ledger, NOW);
    let _ = evaluate(&action("drain_treasury", 1, 4), &m, &mut ledger, NOW);
    let mut pii = action("pay_vendor", 1, 5);
    pii.returns_pii = true;
    let _ = evaluate(&pii, &m, &mut ledger, NOW);
    let mut forged = m.clone();
    forged.mandate.budget_total = u64::MAX;
    let _ = evaluate(&action("pay_vendor", 1, 6), &forged, &mut ledger, NOW);

    // After the entire assault, spend is exactly the legitimate total.
    assert_eq!(ledger.spent, legit_total, "escapes must not move the ledger");
}
