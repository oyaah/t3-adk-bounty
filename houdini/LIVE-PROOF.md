# LIVE proof — Houdini running end-to-end on Terminal 3 testnet

This is the differentiator: most submissions run in demo/seeded mode and never
deploy to a real tenant. Houdini's contract is **registered to a live T3 tenant
and executed inside the real remote TEE**. Reproduce with:

```bash
cd houdini/agent
cp .env.example .env   # put your AGENT_KEY (T3 developer key) in it
set -a; . ./.env; set +a
npx tsx src/deploy.ts
```

## Captured run

```
[1] authenticated  did=did:t3n:dc851f7daab01b36a986b212e49673c2bc00f904  addr=0xe74b1d4442f84a60a79ec684e6b4d700ac1353a3
[3] registering houdini-guard@0.1.0 (217141 bytes wasm)…
[3] registered: {"name":"z:dc851f7daab01b36a986b212e49673c2bc00f904:houdini-guard","contract_id":17}
[4] script z:dc851f7daab01b36a986b212e49673c2bc00f904:houdini-guard @ 0.1.0

  HOUDINI — LIVE on Terminal 3 TEE  contract z:dc851f7daab01b36a986b212e49673c2bc00f904:houdini-guard

  legit   ✓ ALLOW   pay vendor invoice           spent=200
  legit   ✓ ALLOW   pay SaaS subscription        spent=300
  ATTACK  ✗ BLOCKED budget-bust (9999 > cap)     over_per_tx_cap
  ATTACK  ✗ BLOCKED nonce replay (reuse 1)       replay_rejected
  ATTACK  ✗ BLOCKED forged mandate (tampered)    forged_mandate
  ATTACK  ✗ BLOCKED scope escalation (drain)     action_not_permitted
  ATTACK  ✗ BLOCKED PII exfil                    pii_forbidden

  5/5 escapes blocked by the live TEE
```

## What this proves

- **Real Eth sign-in:** `T3nClient.handshake()` + `authenticate(createEthAuthInput(addr))` via the `EthSign` handler — address derived from the developer key, no hardcoding.
- **Real contract on a real tenant:** our `wasm32-wasip2` component registered as `contract_id 17` under `z:<tid>:houdini-guard`.
- **Real TEE enforcement:** every ALLOW/BLOCKED verdict above is returned by the remote enclave executing our contract — the mandate is enforced where the agent cannot reach.

## Honest notes

- The ledger is currently passed per-call (`prior_spent`, `used_nonces`); the
  next step is persisting it in `host:interfaces/kv-store` so cumulative budget
  and replay state survive across calls inside the enclave. Per-call enforcement
  (signature, scope, per-tx cap, PII, and replay against supplied state) is real
  and live.
- `tenant.claim()` returns HTTP 500 on this testnet (logged as a bug); contract
  registration and execution work regardless, so claim is not on the critical path.
