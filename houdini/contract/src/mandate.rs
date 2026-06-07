//! The mandate: a user-signed grant that bounds what the agent may do.
//!
//! The owner signs the canonical bytes of a `Mandate` with their Ed25519 key.
//! The TEE contract verifies that signature before honoring any action — an
//! unsigned or tampered mandate is rejected (`BlockedReason::ForgedMandate`).

use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};

/// The bounded authority granted to the agent. Field order is fixed so the
/// canonical serialization is deterministic on both signer and verifier sides.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Mandate {
    /// Stable id; also namespaces this mandate's ledger + nonce set.
    pub mandate_id: String,
    /// Owner's Ed25519 public key, hex-encoded (32 bytes -> 64 hex chars).
    pub owner_pub_hex: String,
    /// Maximum value of any single action.
    pub per_tx_cap: u64,
    /// Maximum cumulative value across all actions under this mandate.
    pub budget_total: u64,
    /// The only action names the agent may perform.
    pub allowed_actions: Vec<String>,
    /// Unix seconds after which the mandate is dead.
    pub expiry_unix: u64,
}

impl Mandate {
    /// Deterministic bytes the owner signs and the contract verifies over.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        // serde_json preserves struct field declaration order, giving a stable
        // canonical form without pulling in a canonical-JSON dependency.
        serde_json::to_vec(self).expect("Mandate is always serializable")
    }

    pub fn verifying_key(&self) -> Result<VerifyingKey, &'static str> {
        let bytes = hex::decode(&self.owner_pub_hex).map_err(|_| "owner_pub_hex not hex")?;
        let arr: [u8; 32] = bytes.as_slice().try_into().map_err(|_| "owner pub not 32 bytes")?;
        VerifyingKey::from_bytes(&arr).map_err(|_| "owner pub invalid")
    }
}

/// A mandate plus the owner's signature over its canonical bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedMandate {
    pub mandate: Mandate,
    /// Ed25519 signature, hex-encoded (64 bytes -> 128 hex chars).
    pub signature_hex: String,
}

impl SignedMandate {
    /// True only when the signature verifies against the mandate's own owner key.
    pub fn signature_valid(&self) -> bool {
        let Ok(vk) = self.mandate.verifying_key() else {
            return false;
        };
        let Ok(sig_bytes) = hex::decode(&self.signature_hex) else {
            return false;
        };
        let Ok(sig_arr): Result<[u8; 64], _> = sig_bytes.as_slice().try_into() else {
            return false;
        };
        let sig = Signature::from_bytes(&sig_arr);
        vk.verify_strict(&self.mandate.canonical_bytes(), &sig).is_ok()
    }
}
