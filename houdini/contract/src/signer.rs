//! Owner-side signing helpers — used by the demo binary and tests to mint a
//! genuinely-signed mandate. In production the owner signs in their wallet; the
//! contract only ever *verifies* (see `mandate::SignedMandate::signature_valid`).

use crate::mandate::{Mandate, SignedMandate};
use ed25519_dalek::{Signer, SigningKey};

pub fn new_owner() -> SigningKey {
    SigningKey::generate(&mut rand::rngs::OsRng)
}

/// Sign a mandate with the owner's key, producing the verifiable `SignedMandate`.
pub fn sign_mandate(mandate: Mandate, signer: &SigningKey) -> SignedMandate {
    let sig = signer.sign(&mandate.canonical_bytes());
    SignedMandate {
        mandate,
        signature_hex: hex::encode(sig.to_bytes()),
    }
}
