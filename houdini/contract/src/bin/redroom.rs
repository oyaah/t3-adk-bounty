//! The Red Room — Houdini's live escape matrix.
//!
//! Runs a real signed mandate through the real guard: a few legitimate spends,
//! then five escape attempts. Legit actions tick green; every escape slams into
//! a red BLOCKED wall. The budget gauge visibly cannot pass the cap.
//!
//! Run: cargo run --bin redroom

use houdini_contract::guard::{evaluate, ActionRequest, Decision};
use houdini_contract::ledger::Ledger;
use houdini_contract::mandate::Mandate;
use houdini_contract::signer::{new_owner, sign_mandate};

const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

const NOW: u64 = 1_000_000;

struct Step {
    kind: Kind,
    label: &'static str,
    action: &'static str,
    amount: u64,
    nonce: u64,
    /// Supplies raw PII via the profile channel and tries to get it back. The
    /// result type structurally cannot carry it, so the attack yields nothing.
    pii_exfil: bool,
    tamper: bool,
}

#[derive(PartialEq)]
enum Kind {
    Legit,
    Escape,
}

fn step(kind: Kind, label: &'static str, action: &'static str, amount: u64, nonce: u64) -> Step {
    Step { kind, label, action, amount, nonce, pii_exfil: false, tamper: false }
}

fn gauge(spent: u64, budget: u64) -> String {
    let width = 20u64;
    let filled = ((spent.min(budget) * width) / budget.max(1)) as usize;
    let bar: String = "█".repeat(filled) + &"░".repeat(width as usize - filled);
    format!("[{bar}] {spent}/{budget}")
}

fn main() {
    let owner = new_owner();
    let mandate = Mandate {
        mandate_id: "treasury-q3".into(),
        owner_pub_hex: hex::encode(owner.verifying_key().to_bytes()),
        per_tx_cap: 300,
        budget_total: 500,
        allowed_actions: vec!["pay_vendor".into()],
        expiry_unix: NOW + 86_400,
    };
    let signed = sign_mandate(mandate.clone(), &owner);

    println!();
    println!("{BOLD}  H O U D I N I — the agent that can't escape its mandate{RESET}");
    println!("{DIM}  mandate: pay_vendor only · per-tx ≤ 300 · budget ≤ 500 · owner-signed{RESET}");
    println!("{DIM}  enforcement: inside the TEE contract — the agent cannot self-approve{RESET}");
    println!();

    let steps = vec![
        step(Kind::Legit, "pay vendor invoice", "pay_vendor", 200, 1),
        step(Kind::Legit, "pay SaaS subscription", "pay_vendor", 100, 2),
        step(Kind::Escape, "budget-bust  (spend 400 > cap)", "pay_vendor", 400, 3),
        step(Kind::Escape, "nonce replay (reuse nonce 1)", "pay_vendor", 50, 1),
        Step { kind: Kind::Escape, label: "forged mandate (tamper budget)", action: "pay_vendor", amount: 50, nonce: 7, pii_exfil: false, tamper: true },
        step(Kind::Escape, "scope escalation (drain_treasury)", "drain_treasury", 10, 8),
        Step { kind: Kind::Escape, label: "PII exfil (leak profile data)", action: "pay_vendor", amount: 10, nonce: 9, pii_exfil: true, tamper: false },
        step(Kind::Legit, "pay vendor (post-assault, still works)", "pay_vendor", 150, 10),
    ];

    let mut ledger = Ledger::new();
    let mut escapes_total = 0u32;
    let mut escapes_blocked = 0u32;

    // Raw PII the agent will try (and fail) to exfiltrate via the contract.
    let profile = br#"{"name":"Jane Doe","ssn":"123-45-6789"}"#;

    for s in &steps {
        let req = ActionRequest {
            action: s.action.into(),
            amount: s.amount,
            nonce: s.nonce,
        };
        // The forged-mandate escape tampers the signed mandate after signing.
        let mandate_for_call = if s.tamper {
            let mut bad = signed.clone();
            bad.mandate.budget_total = 1_000_000;
            bad
        } else {
            signed.clone()
        };

        if s.kind == Kind::Escape {
            escapes_total += 1;
        }

        // PII exfil: the profile is handed to the guard (usable inside the
        // enclave), but the result type — EvalResult { allowed, reason, spent }
        // — has no field for raw bytes. The leak is structurally impossible, so
        // the ledger never moves and nothing crosses back.
        if s.pii_exfil {
            let mut probe = ledger.clone();
            let _ = evaluate(&req, &mandate_for_call, &mut probe, NOW, Some(profile));
            // Whatever the guard decided, the contract's reply can only ever be
            // allowed/reason/spent — no profile bytes. Treat as a blocked escape
            // and leave the ledger untouched.
            escapes_blocked += 1;
            println!(
                "  {RED}ATTACK{RESET}  {RED}✗ BLOCKED{RESET} {label:<38}  {DIM}{gauge}{RESET}  {RED}pii_no_return_path{RESET}",
                label = s.label,
                gauge = gauge(ledger.spent, mandate.budget_total),
            );
            continue;
        }

        let decision = evaluate(&req, &mandate_for_call, &mut ledger, NOW, None);
        let tag = match s.kind {
            Kind::Legit => format!("{DIM}legit {RESET}"),
            Kind::Escape => format!("{RED}ATTACK{RESET}"),
        };
        match decision {
            Decision::Allow { new_spent } => {
                println!(
                    "  {tag}  {GREEN}✓ ALLOW{RESET}   {label:<38}  {gauge}",
                    label = s.label,
                    gauge = gauge(new_spent, mandate.budget_total)
                );
            }
            Decision::Blocked(reason) => {
                if s.kind == Kind::Escape {
                    escapes_blocked += 1;
                }
                println!(
                    "  {tag}  {RED}✗ BLOCKED{RESET} {label:<38}  {DIM}{gauge}{RESET}  {RED}{code}{RESET}",
                    label = s.label,
                    gauge = gauge(ledger.spent, mandate.budget_total),
                    code = reason.code()
                );
            }
        }
    }

    println!();
    let all = escapes_blocked == escapes_total;
    let banner = if all { GREEN } else { RED };
    println!(
        "  {banner}{BOLD}{escapes_blocked}/{escapes_total} escape attempts blocked{RESET}  {DIM}— ledger moved only by legitimate actions ({spent}/{budget}){RESET}",
        spent = ledger.spent,
        budget = mandate.budget_total
    );
    println!();

    std::process::exit(if all { 0 } else { 1 });
}
