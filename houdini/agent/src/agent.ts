// The Houdini agent: connects to T3, then proposes actions that are evaluated
// by the REAL contract guard (the Rust `eval` binary). The agent cannot approve
// itself — every decision comes back from the enclave logic.

import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";
import { existsSync } from "node:fs";
import { connect } from "./t3.js";
import { newOwner, signMandate, type SignedMandate } from "./mandate.js";

const __dir = dirname(fileURLToPath(import.meta.url));

export interface ActionRequest {
  action: string;
  amount: number;
  nonce: number;
  returns_pii?: boolean;
}

export interface EvalPayload {
  signed: SignedMandate;
  request: ActionRequest;
  prior_spent?: number;
  used_nonces?: number[];
  now_unix: number;
}

export interface EvalResult {
  allowed: boolean;
  reason: string | null;
  spent: number;
}

/** Path to the compiled Rust evaluator (host release build). */
export function evalBinPath(): string {
  return resolve(__dir, "../../contract/target/release/eval");
}

/** Send a payload through the genuine contract guard. Throws if the bin is absent. */
export function callContract(payload: EvalPayload): EvalResult {
  const bin = evalBinPath();
  if (!existsSync(bin)) {
    throw new Error(
      `contract eval binary not found at ${bin} — run: cargo build --release --manifest-path houdini/contract/Cargo.toml`,
    );
  }
  const out = spawnSync(bin, { input: JSON.stringify(payload), encoding: "utf8" });
  if (out.status !== 0) {
    throw new Error(`eval failed: ${out.stderr || out.stdout}`);
  }
  return JSON.parse(out.stdout.trim()) as EvalResult;
}

const NOW = 1_000_000;

/** `npm run demo`: connect, sign a mandate in TS, prove the bridge end to end. */
async function main() {
  const conn = await connect();
  console.log(`\n  Houdini agent connected: ${conn.did}  (${conn.mode} mode)\n`);

  const owner = newOwner();
  const signed = signMandate(
    {
      mandate_id: "treasury-q3",
      per_tx_cap: 300,
      budget_total: 500,
      allowed_actions: ["pay_vendor"],
      expiry_unix: NOW + 86_400,
    },
    owner,
  );

  // Legit action -> ALLOW from the real contract.
  const ok = callContract({
    signed,
    request: { action: "pay_vendor", amount: 200, nonce: 1 },
    now_unix: NOW,
  });
  console.log(`  legit pay_vendor 200 -> allowed=${ok.allowed} spent=${ok.spent}`);

  // Escape attempt (scope) -> BLOCKED.
  const blocked = callContract({
    signed,
    request: { action: "drain_treasury", amount: 10, nonce: 2 },
    now_unix: NOW,
  });
  console.log(`  escape drain_treasury -> allowed=${blocked.allowed} reason=${blocked.reason}`);

  console.log(`\n  Run the full visual matrix: cargo run --bin redroom --manifest-path houdini/contract/Cargo.toml\n`);
}

// Run only when invoked directly (not when imported by tests).
if (process.argv[1] && process.argv[1].endsWith("agent.ts")) {
  main().catch((e) => {
    console.error(e);
    process.exit(1);
  });
}
