// Cross-language bridge proof: a mandate signed in TypeScript (Node Ed25519)
// must verify inside the Rust TEE contract, and a tampered one must be rejected.
// This proves the agent talks to the genuine guard — not a mock.

import { describe, it, expect } from "vitest";
import { existsSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { newOwner, signMandate } from "../src/mandate.js";
import { callContract, evalBinPath, type EvalPayload } from "../src/agent.js";

const NOW = 1_000_000;
const hasBin = existsSync(evalBinPath());

/** A fresh enclave state dir per scenario, so the persistent ledger is clean. */
const freshState = () => mkdtempSync(join(tmpdir(), "houdini-test-"));

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
    const r = callContract(payload, freshState());
    expect(r.allowed).toBe(true);
    expect(r.spent).toBe(200);
  });

  it("a tampered mandate is rejected as forged", () => {
    const signed = legitMandate();
    signed.mandate.budget_total = 1_000_000; // tamper after signing
    const r = callContract(
      {
        signed,
        request: { action: "pay_vendor", amount: 200, nonce: 1 },
        now_unix: NOW,
      },
      freshState(),
    );
    expect(r.allowed).toBe(false);
    expect(r.reason).toBe("forged_mandate");
  });

  it("scope escalation is blocked by the Rust guard", () => {
    const r = callContract(
      {
        signed: legitMandate(),
        request: { action: "drain_treasury", amount: 10, nonce: 2 },
        now_unix: NOW,
      },
      freshState(),
    );
    expect(r.allowed).toBe(false);
    expect(r.reason).toBe("action_not_permitted");
  });

  it("replay + cumulative budget are enforced ACROSS calls by the enclave store", () => {
    // Same mandate, same persistent state dir: the ledger survives between calls
    // and the agent cannot reset it — there is no ledger field in the payload.
    const signed = legitMandate();
    const state = freshState();
    const call = (amount: number, nonce: number) =>
      callContract({ signed, request: { action: "pay_vendor", amount, nonce }, now_unix: NOW }, state);

    expect(call(200, 1).allowed).toBe(true); // spent 200
    // Replay nonce 1 (<= watermark) — rejected even though the payload is "fresh".
    expect(call(50, 1).reason).toBe("replay_rejected");
    // Cumulative budget: 200 + 300 + 300 would exceed 500 on the 3rd.
    expect(call(300, 2).allowed).toBe(true); // spent 500
    expect(call(300, 3).reason).toBe("over_budget");
  });
});
