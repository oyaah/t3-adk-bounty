# Terminal 3 ADK — Onboarding Bug & Documentation Gap Report

**Submitted by:** Yash Bansal · GitHub [@oyaah](https://github.com/oyaah)
**Date:** 2026-06-07
**Track:** $200 Bug Discovery Bounty (Terminal 3 ADK Bounty Challenge, beta)
**Scope:** Onboarding journey + ADK docs (`docs.terminal3.io`) cross-checked against the **shipped SDK** (`@terminal3/t3n-sdk@3.5.0`, type defs) and the **official sample** (`github.com/Terminal-3/z-tenant-flight`).
**Method:** Walked the documented onboarding path on macOS (Node 25, Rust 1.96, `wasm32-wasip2`). Installed the real SDK + toolchain, built the official sample contract, and diffed every doc example against shipped `.d.ts` exports and the sample's real WIT/Cargo/README.

## Verification status
- ✅ Rust toolchain + `wasm32-wasip2` installed; **official sample builds clean** (`cargo build --target wasm32-wasip2 --release` → `z_tenant_flight.wasm`, 198 KB).
- ✅ `@terminal3/t3n-sdk@3.5.0` installed; exports inspected from `dist/index.d.ts`.
- ⏳ Live invocation against testnet pending valid key (runtime section D).

## Summary

| Severity | Count |
|----------|-------|
| Critical | 4 |
| High | 9 |
| Medium | 7 |
| Low | 4 |
| **Total** | **24** |

**Headline findings (proof-grade):**
1. Docs say *"initialize a `T3nClient` with your API key"* — the shipped `T3nClientConfig` has **no `apiKey` field** (requires `wasmComponent`). The documented auth model is wrong.
2. Docs call `client.setEnvironment("testnet")` — `setEnvironment` is a **module-level function**, not a client method.
3. The contract WIT example in the docs uses namespaces/versions (`t3n:host/*@0.1.0`, `export t3n:contract/dispatch`) that **do not exist** in the real host — the real sample uses `host:interfaces/*@2.1.0` and `export contracts`. Code copied from docs cannot load.
4. The numbered walkthrough (Write→Build→Register→Invoke) **omits the mandatory Create-Maps + Seed-Secrets + host-capability-manifest steps** — onboarding cannot complete by following it.

---

## E. Doc-vs-shipped-code contradictions (proof-grade)

### BUG-01 — [CRITICAL] `T3nClient` does not take an API key
- **Doc:** set-up-dev-env Step 4 — *"Initialize a `T3nClient` with your API key."*
- **Reality (`dist/index.d.ts` L1244):** `interface T3nClientConfig { baseUrl?; wasmComponent: WasmComponent; transport?; timeout?; headers?; logLevel?; logger?; handlers? }`. **No `apiKey`.** `wasmComponent` is required. Auth happens via Eth-signing handlers (`metamask_sign`)/`createEthAuthInput`, not an API key on the client.
- **Impact:** First SDK step as documented is impossible; misrepresents the whole auth model.
- **Fix:** Rewrite Step 4 to: `loadWasmComponent()` → `new T3nClient({ wasmComponent, handlers })` → `authenticate(createEthAuthInput(addr))`.
- **Severity:** Critical.

### BUG-02 — [CRITICAL] `setEnvironment` is module-level, not a client method
- **Doc:** set-up-dev-env Step 4 — *"call `setEnvironment('testnet')`"* in T3nClient context.
- **Reality (L2934):** `declare function setEnvironment(env: Environment): void;` — a standalone export. `T3nClient` has no `setEnvironment` method. `client.setEnvironment(...)` → `TypeError: not a function`.
- **Fix:** `import { setEnvironment } from "@terminal3/t3n-sdk"; setEnvironment("testnet");`
- **Severity:** Critical.

### BUG-03 — [CRITICAL] Docs contract WIT uses non-existent host interfaces
- **Doc:** write-contract `world.wit` — `package acme:travel-contract@0.1.0; import t3n:host/kv-store@0.1.0; import t3n:host/logging@0.1.0; import t3n:host/tenant-context@0.1.0; ... export t3n:contract/dispatch@0.1.0;`
- **Reality (sample `wit/world.wit`):** `import host:tenant/tenant-context@1.0.0; import host:interfaces/logging@2.1.0; import host:interfaces/kv-store@2.1.0; import host:interfaces/http@2.1.0; import host:interfaces/http-with-placeholders@2.1.0; export contracts;`
- **Impact:** Namespaces (`t3n:host/*` vs `host:interfaces/*`), versions (`0.1.0` vs `2.1.0`), and the export (`t3n:contract/dispatch` vs `contracts`) are all wrong. Docs themselves say *"the host refuses to load your contract if it imports unavailable interfaces"* — so a docs-following contract is guaranteed to fail to load.
- **Severity:** Critical.

### BUG-04 — [CRITICAL] Onboarding flow omits mandatory steps
- **Doc:** numbered walkthrough = 1 Write → 2 Build → 3 Register → 4 Invoke.
- **Reality (sample README + create-kv-maps + seed-api-key):** before invocation you MUST (a) declare a `host_capabilities` manifest, (b) create the `secrets` KV map with reader/writer ACLs, (c) seed `duffel_api_key`, (d) grant agent egress/auth. None are in the walkthrough; create-kv-maps states *"the map must exist before the contract can run."*
- **Repro:** follow walkthrough verbatim with the sample → invoke → `map not found` / `CredentialsNotConfigured`.
- **Fix:** add numbered steps for manifest, map creation, secret seeding, and egress grant, in order.
- **Severity:** Critical.

### BUG-05 — [HIGH] Docs dispatch model contradicts real contract ABI
- **Doc:** write-contract — `impl Guest { fn dispatch(input: ContractInput) -> Result<ContractOutput, ContractError> }` routing on `input.function`.
- **Reality (sample WIT):** export is `interface contracts` exposing `search-offers`/`book-offer` directly, with a `generic-input { input, user-profile, context }` envelope — no `dispatch`, no `ContractInput`/`ContractOutput` types.
- **Impact:** Rust copied from docs won't compile against the real generated bindings.
- **Severity:** High.

### BUG-06 — [HIGH] Two different APIs documented for seeding a secret
- **Docs:** tips/seed-api-key → `tenant.executeControl("map-entry-set", { map_name: tenant.canonicalName("secrets"), key, value })`. Sample README → `z_sdk.kv("secrets").set("duffel_api_key", ...)`.
- **Reality:** SDK exposes `executeControl`/`canonicalName` (L3171–3173); there is no `kv().set()` surface. README example is non-functional / pseudocode but presented as runnable.
- **Severity:** High.

### BUG-07 — [HIGH] Secret-map naming mechanism contradicts itself
- **Docs:** write-contract — *"KV operations use map tails only; pass `duffel_api_key`, host handles `z:<tid>:` prefixing."*
- **Sample WIT note:** *"The contract builds the full map name at runtime using `tenant-context.tenant-did()`."* i.e. the contract constructs `z:<tid>:secrets` itself, not auto-prefixed.
- **Impact:** Contradictory guidance on who prefixes the namespace → wrong key, `map not found`.
- **Severity:** High.

### BUG-08 — [MEDIUM] Cargo.toml in docs omits `default-features = false`
- **Docs:** `wit-bindgen = { version = "0.49", features = ["macros", "realloc"] }`, `serde_json = { version = "1.0", features = ["alloc"] }`.
- **Sample:** all deps use `default-features = false` (needed for `no_std`/alloc WASM). Docs version pulls std features that can break the `wasm32-wasip2` build.
- **Severity:** Medium.

### BUG-09 — [MEDIUM] `host_capabilities` manifest never documented
- **Sample README:** requires `{ "host_capabilities": ["kv_store","logging","tenant_context","http"] }`. No ADK doc page mentions this manifest or where it lives.
- **Severity:** Medium.

### BUG-10 — [LOW] Sample internal version drift
- `README.md` title says "v0.3.0", `Cargo.toml` is `0.4.1`, `world.wit` is `@0.4.0`. Three different versions in one repo.
- **Severity:** Low.

---

## A. Onboarding flow / walkthrough gaps

### BUG-11 — [HIGH] Invoke walkthrough uses helpers with no import
- **Page:** invoke-contract. Uses `getNodeUrl()`, `getScriptVersion()`, `createEthAuthInput()`, `metamask_sign()`.
- **Reality:** all four ARE real exports (L2948/2768/332/2047) — but the example never shows the `import { ... } from "@terminal3/t3n-sdk"` line, so copy-paste → `ReferenceError`.
- **Fix:** add the import statement.
- **Severity:** High.

### BUG-12 — [HIGH] `authenticate()` requires an argument the setup omits
- **Reality (L1539):** `authenticate(authInput: AuthInput): Promise<Did>`. set-up-dev-env prose implies a bare `authenticate()`. Must pass `createEthAuthInput(address)`.
- **Severity:** High.

### BUG-13 — [HIGH] `loadWasmComponent` flow missing from setup
- **Reality (L249):** `loadWasmComponent(config?): Promise<WasmComponent>` is module-level and its result is **required** by `new T3nClient({ wasmComponent })`. Setup says only "load the WASM component" with no name, no import, no wiring into the constructor.
- **Severity:** High.

### BUG-14 — [HIGH] Register-before-maps ordering never stated
- `maps.create` needs `{ only: [contractId] }`; `contractId` only exists after `register()`. Required order (register → maps → seed → invoke) is documented nowhere; create-kv-maps shows `contractId` undefined.
- **Severity:** High.

### BUG-15 — [MEDIUM] Agent/user clients used without setup
- invoke-contract uses `userClient`, `agentClient`, `agentDid`, `agentAddress`, `agentKey`; setup only builds one `TenantClient`. Creation of the agent identity/key and user client is never shown.
- **Severity:** Medium.

---

## B. Documentation gaps in examples

### BUG-16 — [HIGH] Two documented errors have no fix
- **Page:** common-errors — `email_not_verified` and `user_not_found` rows have blank resolutions ("fix missing").
- **Severity:** High.

### BUG-17 — [MEDIUM] `ContractError` type never defined/imported
- write-contract returns `ContractError::*` variants; never imported or enumerated. Doc itself admits *"contract-authored errors are undefined beyond examples."*
- **Severity:** Medium.

### BUG-18 — [MEDIUM] seed-api-key snippet references undefined symbols
- Uses `tenant.executeControl`/`tenant.canonicalName` with no import/introduction (they exist at L3171/3173 but are never shown being obtained).
- **Severity:** Medium.

### BUG-19 — [MEDIUM] Payroll-agent use-case page is an empty stub
- use-cases/payroll-agent has only a header + one link; no code or explanation.
- **Severity:** Medium.

### BUG-20 — [MEDIUM] snake_case vs camelCase API drift
- `register()` returns `result.contract_id` (snake); create-kv-maps consumes `contractId` (camel). No bridging assignment shown.
- **Severity:** Medium.

---

## C. Consistency / clarity / tooling

### BUG-21 — [LOW] Duplicated toolchain command
- `rustup target add wasm32-wasip2` appears in both set-up-dev-env Step 2 and build-contract, the latter framing it as part of every build.
- **Severity:** Low.

### BUG-22 — [LOW] Claim URL naming + no key recovery
- "terminal3.io/claim-page" vs "tokens claim page"; key "shown once, can't be retrieved" with no documented reissue path.
- **Severity:** Low.

### BUG-23 — [LOW] `readers must be set explicitly` footgun only in tips
- KV governor "defaults to deny"; omitting `readers` → silent `AccessDenied`. Warning lives only in tips, not the (missing) walkthrough map step.
- **Severity:** Low.

### BUG-24 — [MEDIUM] SDK ships a transitive vuln with no fix available
- **Repro:** `npm install @terminal3/t3n-sdk` → `npm audit` → 3 moderate.
- **Chain:** `@terminal3/t3n-sdk@3.5.0` → `ethers (>=6.0.0-beta.1)` → `ws@8.0.0–8.20.0`.
- **Advisory:** `ws` Uninitialized memory disclosure — **GHSA-58qx-3vcg-4xpx**. npm reports **"No fix available"** (no patched `ws` in the range `ethers` pins).
- **Why it matters:** the ADK's entire value prop is security/privacy; shipping a known unpatched memory-disclosure transitive dep undercuts that and is silent during onboarding.
- **Fix:** bump `ethers` to a release pinning `ws > 8.20.0`, or add an `overrides`/`resolutions` entry + note it in the install docs.
- **Severity:** Medium.

---

## D. Live runtime findings (fill during your own run)

> Run the STEPS in PLAN.md with a valid key. Paste exact terminal errors here — each upgrades a finding from "verified against shipped code" to "reproduced live end-to-end."

### BUG-25 —
- **Step:** (setup / write / build / register / maps / seed / invoke)
- **Command/code:**
- **Expected:**
- **Actual (paste exact error):**
- **Severity:**
- **Screenshot:**

<!-- duplicate for each new finding -->
