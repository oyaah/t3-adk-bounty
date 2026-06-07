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

- ✅ **LIVE end-to-end on Terminal 3 testnet** — real Eth handshake + authenticate, our `wasm32-wasip2` contract **registered to a real tenant** (`contract_id 17`, `z:<tid>:houdini-guard`), and **executed inside the real remote TEE**: 2 legit ALLOW, 5/5 escapes BLOCKED by the enclave. Full capture in [`LIVE-PROOF.md`](./LIVE-PROOF.md). This is the gap most submissions have — they run seeded/demo-mode and never deploy.
- ✅ **Real:** the contract + fail-closed guard, Ed25519 mandate signing/verification (TS signer verified inside the Rust contract), the 5 enforced rejections, real `@terminal3/t3n-sdk` auth (no mocks, no hardcoded address — derived from the key).
- 🟡 **One honest limitation:** the ledger is currently passed per-call (`prior_spent`, `used_nonces`); persisting it in `host:interfaces/kv-store` so cumulative budget + replay state live inside the enclave across calls is the next step. Per-call enforcement (signature, scope, per-tx cap, PII, replay vs supplied state) is real and live today.

No theater: auth, deploy, and enforcement are genuine and reproducible against the live network.

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

# LIVE end-to-end on T3 testnet (real tenant, real TEE)
cp .env.example .env            # add your AGENT_KEY (T3 developer key)
set -a; . ./.env; set +a
npx tsx src/deploy.ts           # registers the contract + runs the matrix on the live enclave
```

See [`DEMO.md`](./DEMO.md) for the 90-second demo script.
