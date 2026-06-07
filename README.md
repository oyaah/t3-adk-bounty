<div align="center">

# 🪤 Houdini + T3 ADK Bug Report

### Terminal 3 ADK Bounty Challenge — both tracks, one repo

**Houdini** is the AI agent that tries to escape its own limits — and *can't*, because its mandate is enforced **inside a real Terminal 3 TEE**, where the agent can't reach.

[🎬 Demo](houdini/DEMO.md) · [🪤 Houdini ($300 agent track)](houdini/README.md) · [🐞 Bug Report ($200 track)](BUGLOG.md) · [✅ Live TEE proof](houdini/LIVE-PROOF.md)

</div>

---

## What's in this repo

| Track | What | Where |
|---|---|---|
| **$300 — Best Agent** | **Houdini** — a self-attacking agent whose mandate is enforced in a TEE contract | [`houdini/`](houdini/) |
| **$200 — Bug Bounty** | Onboarding bug + doc-gap report against the live docs & shipped SDK, re-verified on submission day | [`BUGLOG.md`](BUGLOG.md) · [`PROOF.md`](PROOF.md) · [`FIXED-ONBOARDING.md`](FIXED-ONBOARDING.md) |

---

## 🪤 Houdini — the one-liner

> Every other agent **says** it respects its limits. Houdini **can't exceed them** — and it proves it by attacking itself, every run, in CI.

Letting an AI agent touch money or data needs exactly one guarantee: **it cannot overstep what you authorized.** Today that guarantee is usually theater — limits are issued as credentials but never *checked at execution*. Houdini makes it **structural**: a user-signed mandate (per-tx cap, total budget, allowed actions, expiry) is enforced inside a Rust→WASM contract running in the Terminal 3 TEE. The agent proposes; the enclave decides. **The agent cannot self-approve.**

### The escape matrix — 5 attacks, all blocked by the enclave

```
  legit   ✓ ALLOW   pay vendor invoice                      [████████░░░░░░░░░░░░] 200/500
  legit   ✓ ALLOW   pay SaaS subscription                   [████████████░░░░░░░░] 300/500
  ATTACK  ✗ BLOCKED budget-bust  (spend 400 > cap)          [████████████░░░░░░░░] 300/500  over_per_tx_cap
  ATTACK  ✗ BLOCKED nonce replay (reuse nonce 1)            [████████████░░░░░░░░] 300/500  replay_rejected
  ATTACK  ✗ BLOCKED forged mandate (tamper budget)          [████████████░░░░░░░░] 300/500  forged_mandate
  ATTACK  ✗ BLOCKED scope escalation (drain_treasury)       [████████████░░░░░░░░] 300/500  action_not_permitted
  ATTACK  ✗ BLOCKED PII exfil (leak profile data)           [████████████░░░░░░░░] 300/500  pii_no_return_path
  legit   ✓ ALLOW   pay vendor (post-assault, still works)  [██████████████████░░] 450/500

  5/5 escape attempts blocked — ledger moved only by legitimate actions (450/500)
```

| # | Escape | Why it can't (enforced in TEE) |
|---|--------|--------------------------------|
| 1 | **Budget-bust** — spend past the cap | Guard reads the **enclave-owned** ledger, denies; ledger unchanged |
| 2 | **Nonce replay** — re-use an approved action | Enclave-owned nonce watermark rejects nonce ≤ last |
| 3 | **Forged mandate** — tamper or wrong key | Ed25519 signature check fails |
| 4 | **Scope escalation** — call an action outside the allowlist | Not in mandate → denied |
| 5 | **PII exfil** — hand profile in, try to read it back | Structural: the result type has **no field** for PII |

### The brain can be tricked. The cage can't.
Houdini's planner is an **untrusted LLM** — it can be wrong, jailbroken, or prompt-injected. It doesn't matter: whatever it proposes still has to pass the TEE guard. Security doesn't depend on the model behaving. It depends on the hardware.

### What's real (no theater)
- ✅ **Live on Terminal 3 testnet** — real Eth handshake + authenticate, our `wasm32-wasip2` contract **registered to a real tenant** (`contract_id 17`) and **executed in the real remote TEE**: 2 legit ALLOW, 5/5 escapes BLOCKED. → [`LIVE-PROOF.md`](houdini/LIVE-PROOF.md)
- ✅ **Enclave-owned ledger** — cumulative spend + replay watermark live in the real `host:interfaces/kv-store@2.1.0`, keyed by `mandate_id`. The request payload carries **no** ledger state, so replay + budget can't be faked by the agent.
- ✅ **Real Ed25519 mandate** — signed in TypeScript, verified inside the Rust contract.
- ✅ **Machine-checked** — a 6-test escape matrix asserts every attack is blocked *and the ledger never moves*; CI fails if any escape isn't blocked.

---

## ▶️ Reproduce in 60 seconds

```bash
# 1. Build the TEE contract + prove the export
cargo build --release --target wasm32-wasip2 --manifest-path houdini/contract/Cargo.toml
wasm-tools component wit houdini/contract/target/wasm32-wasip2/release/houdini_contract.wasm | grep export
#   -> export houdini:contract/contracts@0.1.0

# 2. The Red Room — the money shot (2 legit spends, 5 blocked escapes)
cargo run --bin redroom --manifest-path houdini/contract/Cargo.toml

# 3. The proof is machine-checked, not asserted
cargo test --manifest-path houdini/contract/Cargo.toml --target "$(rustc -vV | sed -n 's/host: //p')"

# 4. Live on the real T3 testnet (needs your AGENT_KEY)
cd houdini/agent && npm install && cp .env.example .env   # add AGENT_KEY
npx tsx src/deploy.ts
```

No Rust? Open [`houdini/demo/index.html`](houdini/demo/index.html) in a browser — a self-contained visual replay of the live TEE run.

---

## 🐞 $200 Bug Track

A detailed onboarding bug + doc-gap report against the live `docs.terminal3.io` and the shipped `@terminal3/t3n-sdk@3.5.0`, **re-verified on submission day** (the live docs were corrected for several earlier findings — flagged honestly rather than over-claimed). See [`BUGLOG.md`](BUGLOG.md).

---

<div align="center">
<sub>Built for the Terminal 3 ADK Bounty Challenge · privacy without compromise, hardware-attested mandates, on the real ADK.</sub>
</div>
