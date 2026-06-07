//! The ledger: the mandate's running state — cumulative spend and the set of
//! nonces already consumed. In the production TEE contract this lives in the
//! `host:interfaces/kv-store` map `z:<tid>:houdini:<mandate_id>`; here it is an
//! in-struct model so the enforcement logic is host-testable and the demo is
//! deterministic (see KTD-2: local enclave-faithful harness).
//!
//! The single invariant that makes Houdini un-escapable: the ledger is mutated
//! ONLY on a fully-approved action. Every rejection leaves it byte-for-byte
//! unchanged — proven by the escape-matrix tests.

use std::collections::HashSet;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ledger {
    /// Cumulative value of all approved actions so far.
    pub spent: u64,
    /// Nonces of approved actions — replay guard.
    pub used_nonces: HashSet<u64>,
}

impl Ledger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_nonce_used(&self, nonce: u64) -> bool {
        self.used_nonces.contains(&nonce)
    }

    /// Commit an approved action. Only the guard calls this, and only after
    /// every check has passed.
    pub fn commit(&mut self, nonce: u64, amount: u64) {
        self.used_nonces.insert(nonce);
        self.spent += amount;
    }

    pub fn remaining(&self, budget_total: u64) -> u64 {
        budget_total.saturating_sub(self.spent)
    }
}
