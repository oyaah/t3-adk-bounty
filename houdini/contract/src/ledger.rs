//! The ledger: the mandate's running state — cumulative spend and the
//! high-water nonce that defeats replay. State is OWNED BY THE ENCLAVE, not the
//! caller: it is loaded from and committed to a `LedgerStore` keyed by
//! `mandate_id`. The agent (the untrusted party that builds the request payload)
//! has no field through which to set `spent` or reset the nonce watermark.
//!
//! In the production TEE contract the store is `host:interfaces/kv-store` under
//! the map `z:<tid>:houdini:<mandate_id>`. On the host (tests/CLI) it is a
//! file-backed store under a state dir. Either way it persists across calls and
//! is never read from the request.
//!
//! The single invariant that makes Houdini un-escapable: the ledger is mutated
//! ONLY on a fully-approved action. Every rejection leaves it byte-for-byte
//! unchanged — proven by the escape-matrix tests.

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ledger {
    /// Cumulative value of all approved actions so far.
    pub spent: u64,
    /// Highest nonce ever approved. A new action must use a strictly greater
    /// nonce (x402 monotonic pattern) — so a replayed or stale nonce is rejected
    /// without storing an unbounded set.
    pub last_nonce: u64,
}

impl Ledger {
    pub fn new() -> Self {
        Self::default()
    }

    /// True if `nonce` has already been consumed (or is stale). Replay guard.
    pub fn is_nonce_used(&self, nonce: u64) -> bool {
        nonce <= self.last_nonce
    }

    /// Commit an approved action. Only the guard calls this, and only after
    /// every check has passed.
    pub fn commit(&mut self, nonce: u64, amount: u64) {
        self.last_nonce = nonce;
        self.spent += amount;
    }

    pub fn remaining(&self, budget_total: u64) -> u64 {
        budget_total.saturating_sub(self.spent)
    }
}

/// Enclave-owned persistent state. The caller cannot influence or reset `spent`
/// or the nonce watermark — that is the genuineness property for the replay and
/// cumulative-budget defenses. Implementations persist across calls keyed by
/// `mandate_id` and are NEVER populated from the request payload.
pub trait LedgerStore {
    fn load(&self, mandate_id: &str) -> Ledger;
    fn commit(&self, mandate_id: &str, nonce: u64, amount: u64);
}
