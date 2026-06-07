# Terminal 3 ADK Bounty — Bug Discovery Submission

**Hacker:** Yash Bansal · GitHub [@oyaah](https://github.com/oyaah)
**Track:** $200 Bug Discovery Bounty
**Date:** 2026-06-07

## TL;DR

I installed the real ADK toolchain end-to-end, built the official sample contract, and diffed **every** documentation example against the **shipped SDK** (`@terminal3/t3n-sdk@3.5.0`) and the **official sample** (`z-tenant-flight`). Result: **24 distinct bugs / documentation gaps**, including **4 critical** issues where a developer following the docs literally cannot succeed:

1. **`T3nClient` takes no API key** — docs say to pass one; the shipped `T3nClientConfig` has no such field (requires `wasmComponent`).
2. **`setEnvironment` is a module-level function**, not a `T3nClient` method as documented (`client.setEnvironment` → `TypeError`).
3. **Docs' contract WIT is invalid** — uses `t3n:host/*@0.1.0` + `export t3n:contract/dispatch`; the real host uses `host:interfaces/*@2.1.0` + `export contracts`. Proven against the compiled component.
4. **The onboarding walkthrough omits mandatory steps** (host-capability manifest, KV map creation, secret seeding, egress grant) — invocation fails without them.

Full itemized report with severities, repros, file/line citations, and fixes: **[BUGLOG.md](./BUGLOG.md)**.

## What makes this submission credible

- Every claim is **verifiable**: SDK findings cite `dist/index.d.ts` line numbers; WIT findings are proven against a **compiled `.wasm`** component.
- I reproduced the build path on a clean machine — the official sample compiles, so the failures I report are doc/SDK-surface issues, not environment noise.

## Environment (clean install, macOS arm64)

```
node v25.9.0 | npm 11.12.1 | rustc 1.96.0 | wasm-tools 1.251.0
target: wasm32-wasip2 (installed)
@terminal3/t3n-sdk@3.5.0
```

## Proof artifacts

### Compiled component exports (disproves docs BUG-03)
```
$ wasm-tools component wit target/wasm32-wasip2/release/z_tenant_flight.wasm
world root {
  import host:tenant/tenant-context@1.0.0;
  import host:interfaces/logging@2.1.0;
  import host:interfaces/kv-store@2.1.0;
  import host:interfaces/http@2.1.0;
  import host:interfaces/http-with-placeholders@2.1.0;
  ...
  export z:tenant-flight/contracts@0.4.0;     // docs claim: export t3n:contract/dispatch@0.1.0
}
```

### Shipped SDK config (disproves docs BUG-01)
```
// dist/index.d.ts:1244
interface T3nClientConfig {
  baseUrl?: string;
  wasmComponent: WasmComponent;   // REQUIRED — no apiKey field anywhere
  transport?: Transport;
  ...
}
// dist/index.d.ts:2934
declare function setEnvironment(env: Environment): void;   // module-level, not a method
```

## Repo contents

| File | Purpose |
|---|---|
| `BUGLOG.md` | Full 24-item report (severity, repro, citation, fix) |
| `PLAN.md` | Strategy + step-by-step repro plan |
| `index.mjs` | Onboarding driver stitched from docs (triggers the SDK-surface bugs) |
| `z-tenant-flight/` | Official sample, built locally as the control |

## How to reproduce

```bash
npm install                                  # pulls @terminal3/t3n-sdk@3.5.0 (note: 3 moderate vulns — BUG-24)
rustup target add wasm32-wasip2
cd z-tenant-flight && cargo build --target wasm32-wasip2 --release
wasm-tools component wit target/wasm32-wasip2/release/z_tenant_flight.wasm   # compare exports to docs
node ../index.mjs                            # triggers BUG-01/02/11/12/13 against the SDK surface
```
