# 🪤 Houdini

> **The AI agent that tries to escape its own limits — and can't.**

Houdini runs a real budget under a user-signed **mandate** that is enforced *inside* a Terminal 3 TEE contract. Then it attacks itself five ways trying to break out. Every escape slams into the enclave and bounces.

```
  legit   ✓ ALLOW   pay vendor invoice                      [████████░░░░░░░░░░░░] 200/500
  legit   ✓ ALLOW   pay SaaS subscription                   [████████████░░░░░░░░] 300/500
  ATTACK  ✗ BLOCKED budget-bust  (spend 400 > cap)          [████████████░░░░░░░░] 300/500  over_per_tx_cap
  ATTACK  ✗ BLOCKED nonce replay (reuse nonce 1)            [████████████░░░░░░░░] 300/500  replay_rejected
  ATTACK  ✗ BLOCKED forged mandate (tamper budget)          [████████████░░░░░░░░] 300/500  forged_mandate
  ATTACK  ✗ BLOCKED scope escalation (drain_treasury)       [████████████░░░░░░░░] 300/500  action_not_permitted
  ATTACK  ✗ BLOCKED PII exfil (leak profile data)           [████████████░░░░░░░░] 300/500  pii_forbidden
  legit   ✓ ALLOW   pay vendor (post-assault, still works)  [██████████████████░░] 450/500

  5/5 escape attempts blocked — ledger moved only by legitimate actions (450/500)
```

## The escape matrix

| # | Escape trick | What Houdini tries | Why it can't (enforced in TEE) |
|---|---|---|---|
| 1 | **Budget-bust** | Spend past the per-tx cap / cumulative budget | Guard reads the ledger, denies, ledger unchanged |
| 2 | **Nonce replay** | Re-submit an already-approved action | Nonce ledger marks it spent → `replay_rejected` |
| 3 | **Forged mandate** | Tamper the mandate / sign with the wrong key | Ed25519 signature check fails → `forged_mandate` |
| 4 | **Scope escalation** | Call an action outside the allowlist | Not in mandate → `action_not_permitted` |
| 5 | **PII exfil** | Return raw profile data across the boundary | Refused → `pii_forbidden` (PII never leaves the enclave) |

## Why this matters

Letting an AI agent touch money or private data needs one guarantee: it **cannot overstep what you authorized**. Today that guarantee is usually theater — limits are issued as credentials but never checked at execution, signatures are checked for *presence* not *validity*, "attestation" is a log line.

Houdini makes the guarantee **structural**: the mandate is enforced where the agent can't reach — inside the TEE contract. The agent cannot self-approve. And instead of *claiming* it's safe, Houdini *proves* it by attacking itself, every run, in CI.

This is the working shape of Terminal 3's own thesis — *privacy without compromise*, hardware-attested mandates — built on the real ADK.

## How it works

```
Owner ──signs mandate──▶  TEE contract (Rust→WASM)         Agent (TypeScript)
   per-tx cap, budget,        guard.rs  ── fail-closed ──▶   real @terminal3/t3n-sdk auth
   allowed actions, expiry     ledger (spend + nonces)        proposes actions + 5 self-attacks
                                  │
                                  └──▶  ✓ ALLOW / ✗ BLOCKED + reason  ──▶  Red Room
```

- **Enforcement (`contract/`)** — Rust→WASM (`wasm32-wasip2`) contract exporting `houdini:contract/contracts`. `guard.rs` runs all checks fail-closed and mutates the ledger **only** on full approval.
- **Agent (`agent/`)** — real T3 SDK auth (correct EthSign flow, no mocks), an Ed25519 mandate signer, and a bridge that drives the genuine contract guard.
- **Proof** — 8 contract unit tests + a 6-test **escape matrix** + a cross-language test (a TS-signed mandate verified inside the Rust contract). CI runs all of it and fails if any escape isn't blocked.

## What's live vs simulated (honest box)

- ✅ **Real:** the TEE contract (compiled to `wasm32-wasip2`, verified with `wasm-tools`), the fail-closed guard, Ed25519 mandate signing/verification, the 5 enforced escape rejections, real `@terminal3/t3n-sdk` auth wiring.
- 🟡 **Simulated for the demo (KTD-2):** the ledger is passed in the call payload rather than persisted in `host:interfaces/kv-store` (the production path), and the agent runs in **harness mode** unless `AGENT_KEY`/`AGENT_ADDRESS` are set. Live tenant deployment is the one remaining step — gated on T3's testnet register path (which we documented as buggy; see [`../BUGLOG.md`](../BUGLOG.md)).

No theater: enforcement and the attacks are genuine; only persistence + live deploy are stubbed, and labeled as such.

## Reproduce

```bash
# Contract: tests + escape matrix + the visual demo
cargo test  --manifest-path houdini/contract/Cargo.toml \
            --target "$(rustc -vV | sed -n 's/host: //p')"
cargo run   --bin redroom --manifest-path houdini/contract/Cargo.toml      # the Red Room

# Real TEE artifact
cargo build --release --target wasm32-wasip2 --manifest-path houdini/contract/Cargo.toml
wasm-tools component wit houdini/contract/target/wasm32-wasip2/release/houdini_contract.wasm

# Agent: SDK auth + TS↔contract bridge
cargo build --release --bin eval --manifest-path houdini/contract/Cargo.toml
cd houdini/agent && npm install && npm test
```

See [`DEMO.md`](./DEMO.md) for the 90-second demo script.
