// Cross-language bridge proof: a mandate signed in TypeScript (Node Ed25519)
// must verify inside the Rust TEE contract, and a tampered one must be rejected.
// This proves the agent talks to the genuine guard — not a mock.

import { describe, it, expect } from "vitest";
import { existsSync } from "node:fs";
import { newOwner, signMandate } from "../src/mandate.js";
import { callContract, evalBinPath, type EvalPayload } from "../src/agent.js";

const NOW = 1_000_000;
const hasBin = existsSync(evalBinPath());

function legitMandate() {
  const owner = newOwner();
  return signMandate(
    {
      mandate_id: "treasury-q3",
      per_tx_cap: 300,
      budget_total: 500,
      allowed_actions: ["pay_vendor"],
      expiry_unix: NOW + 86_400,
    },
    owner,
  );
}

describe.skipIf(!hasBin)("TS -> Rust contract bridge", () => {
  it("a TS-signed mandate is honored by the Rust guard (ALLOW)", () => {
    const payload: EvalPayload = {
      signed: legitMandate(),
      request: { action: "pay_vendor", amount: 200, nonce: 1 },
      now_unix: NOW,
    };
    const r = callContract(payload);
    expect(r.allowed).toBe(true);
    expect(r.spent).toBe(200);
  });

  it("a tampered mandate is rejected as forged", () => {
    const signed = legitMandate();
    signed.mandate.budget_total = 1_000_000; // tamper after signing
    const r = callContract({
      signed,
      request: { action: "pay_vendor", amount: 200, nonce: 1 },
      now_unix: NOW,
    });
    expect(r.allowed).toBe(false);
    expect(r.reason).toBe("forged_mandate");
  });

  it("scope escalation is blocked by the Rust guard", () => {
    const r = callContract({
      signed: legitMandate(),
      request: { action: "drain_treasury", amount: 10, nonce: 2 },
      now_unix: NOW,
    });
    expect(r.allowed).toBe(false);
    expect(r.reason).toBe("action_not_permitted");
  });
});
