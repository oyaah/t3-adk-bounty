// Owner-side mandate signing in TypeScript, byte-compatible with the Rust
// contract's `Mandate::canonical_bytes` (serde_json compact, fixed field order).
// Uses Node's native Ed25519 — no extra dependency. The signature this produces
// verifies inside the TEE contract (proven by the cross-language test).

import { generateKeyPairSync, sign as edSign, type KeyObject } from "node:crypto";

export interface Mandate {
  mandate_id: string;
  owner_pub_hex: string;
  per_tx_cap: number;
  budget_total: number;
  allowed_actions: string[];
  expiry_unix: number;
}

export interface SignedMandate {
  mandate: Mandate;
  signature_hex: string;
}

export interface Owner {
  publicKey: KeyObject;
  privateKey: KeyObject;
  pubHex: string;
}

/** Generate an Ed25519 owner keypair and its raw 32-byte public key as hex. */
export function newOwner(): Owner {
  const { publicKey, privateKey } = generateKeyPairSync("ed25519");
  const jwk = publicKey.export({ format: "jwk" }) as { x: string };
  const pubHex = Buffer.from(jwk.x, "base64url").toString("hex");
  return { publicKey, privateKey, pubHex };
}

/**
 * Canonical bytes = serde_json compact encoding with the Rust struct's field
 * order. JSON.stringify over an object with keys inserted in this order yields
 * the identical byte string (no whitespace, integers as-is, ASCII strings).
 */
export function canonicalBytes(m: Mandate): Buffer {
  const ordered = {
    mandate_id: m.mandate_id,
    owner_pub_hex: m.owner_pub_hex,
    per_tx_cap: m.per_tx_cap,
    budget_total: m.budget_total,
    allowed_actions: m.allowed_actions,
    expiry_unix: m.expiry_unix,
  };
  return Buffer.from(JSON.stringify(ordered), "utf8");
}

/** Sign a mandate with the owner's key, producing a contract-verifiable form. */
export function signMandate(
  fields: Omit<Mandate, "owner_pub_hex">,
  owner: Owner,
): SignedMandate {
  const mandate: Mandate = { ...fields, owner_pub_hex: owner.pubHex };
  const sig = edSign(null, canonicalBytes(mandate), owner.privateKey);
  return { mandate, signature_hex: sig.toString("hex") };
}
