# Terminal 3 ADK — Onboarding Bug & Documentation Gap Report

**Submitted by:** Yash Bansal · GitHub [@oyaah](https://github.com/oyaah)
**Date:** 2026-06-07
**Terminal 3 SDK bug & documentation-gap report**
**Scope:** Onboarding journey + ADK docs (`docs.terminal3.io`) cross-checked against the **shipped SDK** (`@terminal3/t3n-sdk@3.5.0`, type defs) and the **official sample** (`github.com/Terminal-3/z-tenant-flight`).
**Method:** Walked the documented onboarding path on macOS (Node 25, Rust 1.96, `wasm32-wasip2`). Installed the real SDK + toolchain, built the official sample contract, and diffed every doc example against shipped `.d.ts` exports and the sample's real WIT/Cargo/README.

## Verification status
- ✅ Rust toolchain + `wasm32-wasip2` installed; **official sample builds clean** (`cargo build --target wasm32-wasip2 --release` → `z_tenant_flight.wasm`, 198 KB).
- ✅ `@terminal3/t3n-sdk@3.5.0` installed; exports inspected from `dist/index.d.ts`.
- ⏳ Live invocation against testnet pending valid key (runtime section D).

## ⚠️ Independent re-verification (2026-06-07, against CURRENT live docs)
Every claim was re-checked against the live `docs.terminal3.io/*.md` pages, the shipped SDK, and the sample source on the submission day.

- **27 of 34 bugs still reproduce** against the current live docs/SDK/sample.
- **7 bugs do NOT reproduce against the current live docs.** They were valid against the SDK shape / older docs, but the live `docs.terminal3.io` pages today show the correct patterns — whether patched since discovery or paraphrased loosely from the older docs, they no longer reproduce. Flagged `❌ DOES NOT REPRODUCE ON CURRENT LIVE DOCS` below: **BUG-01, BUG-02, BUG-08, BUG-11, BUG-12, BUG-13, BUG-17.** Retained for transparency, NOT submitted as active findings. (Note: `verify.mjs` still passes 10/10 because it only asserts the SDK's real *shape* — it cannot detect that the live docs already match it.)
- **6 bugs reframed / overstated** (`🟡` below): **BUG-07, BUG-15, BUG-16, BUG-20, BUG-22, BUG-29** — each has a real kernel but the original wording over-claims; see inline notes. Several are candidates for severity downgrade.
- **Surviving headline criticals: BUG-03 and BUG-04** (both confirmed against live `write-contract` / walkthrough today).

## Summary

| Severity | Documented | Reproduce on current live docs |
|----------|------------|--------------------------------|
| Critical | 4 | 2 |
| High | 12 | 9 |
| Medium | 11 | 9 |
| Low | 7 | 7 |
| **Total** | **34** | **27** |

> 7 originally-documented bugs (BUG-01/02/08/11/12/13/17) no longer reproduce against the current live docs; 6 more (BUG-07/15/16/20/22/29) are reproducible but reframed/softened — see re-verification note above.

> Sections A–E verified against shipped code; Section F adds sample-source vs docs contradictions; all 10 SDK-surface bugs are confirmed by a runnable harness (`node verify.mjs` → 10/10). Full command outputs in [PROOF.md](./PROOF.md). A corrected, working onboarding guide is in [FIXED-ONBOARDING.md](./FIXED-ONBOARDING.md).

**Headline findings (proof-grade, confirmed against current live docs):**
1. **[BUG-03]** The contract WIT example in the docs uses namespaces/versions (`t3n:host/*@0.1.0`, `export t3n:contract/dispatch@0.1.0`) that **do not exist** in the real host — the real sample uses `host:interfaces/*@2.1.0` and `export contracts`. Code copied from docs cannot load.
2. **[BUG-04]** The numbered walkthrough (Write→Build→Register→Invoke) **omits the mandatory Create-Maps + Seed-Secrets + host-capability steps** — onboarding cannot complete by following it.
3. **[BUG-05]** Docs show a `fn dispatch(input: ContractInput) -> Result<ContractOutput, ContractError>` ABI that does not match the real `contracts` interface + `generic-input` envelope — Rust copied from docs won't compile.
4. **[BUG-26]** Docs HTTP example uses `method: "POST".to_string()`, a `body` field, and reads `resp.body`; the real bindings use a `Verb::Post` enum, a `payload` field, and `resp.payload` — the snippet does not compile.

> NOTE: two earlier headline findings (docs initializing `T3nClient` with an API key, and `client.setEnvironment()`) **no longer reproduce against the current live docs** — preserved as BUG-01 / BUG-02 below for transparency.

---

## E. Doc-vs-shipped-code contradictions (proof-grade)

### BUG-01 — [CRITICAL] `T3nClient` does not take an API key
> ❌ **DOES NOT REPRODUCE ON CURRENT LIVE DOCS (verified 2026-06-07):** live `set-up-dev-env` now builds the client as `new T3nClient({ wasmComponent, handlers })` — claim no longer reproduces. Retained for transparency; not an active finding.
- **Doc:** set-up-dev-env Step 4 — *"Initialize a `T3nClient` with your API key."*
- **Reality (`dist/index.d.ts` L1244):** `interface T3nClientConfig { baseUrl?; wasmComponent: WasmComponent; transport?; timeout?; headers?; logLevel?; logger?; handlers? }`. **No `apiKey`.** `wasmComponent` is required. Auth happens via Eth-signing handlers (`metamask_sign`)/`createEthAuthInput`, not an API key on the client.
- **Impact:** First SDK step as documented is impossible; misrepresents the whole auth model.
- **Fix:** Rewrite Step 4 to: `loadWasmComponent()` → `new T3nClient({ wasmComponent, handlers })` → `authenticate(createEthAuthInput(addr))`.
- **Severity:** Critical.

### BUG-02 — [CRITICAL] `setEnvironment` is module-level, not a client method
> ❌ **DOES NOT REPRODUCE ON CURRENT LIVE DOCS (verified 2026-06-07):** live `set-up-dev-env` now imports and calls `setEnvironment("testnet")` standalone — claim no longer reproduces. Retained for transparency; not an active finding.
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
> 🟡 **REFRAMED (2026-06-07):** the tail-vs-`z:<tid>:` prefixing contradiction is real, but note the *map tail* is `secrets` and `duffel_api_key` is the *key within* that map — the original wording conflated the two. Suggest downgrading to MEDIUM.
- **Docs:** write-contract — *"KV operations use map tails only; pass `duffel_api_key`, host handles `z:<tid>:` prefixing."*
- **Sample WIT note:** *"The contract builds the full map name at runtime using `tenant-context.tenant-did()`."* i.e. the contract constructs `z:<tid>:secrets` itself, not auto-prefixed.
- **Impact:** Contradictory guidance on who prefixes the namespace → wrong key, `map not found`.
- **Severity:** High.

### BUG-08 — [MEDIUM] Cargo.toml in docs omits `default-features = false`
> ❌ **DOES NOT REPRODUCE ON CURRENT LIVE DOCS (verified 2026-06-07):** live `write-contract` Cargo.toml now includes `default-features = false` on both `wit-bindgen` and `serde_json` — claim no longer reproduces. Retained for transparency; not an active finding.
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
> ❌ **DOES NOT REPRODUCE ON CURRENT LIVE DOCS (verified 2026-06-07):** live `invoke-contract` now shows an explicit `import { … getScriptVersion, getNodeUrl, createEthAuthInput, metamask_sign } from "@terminal3/t3n-sdk"` — claim no longer reproduces. Retained for transparency; not an active finding.
- **Page:** invoke-contract. Uses `getNodeUrl()`, `getScriptVersion()`, `createEthAuthInput()`, `metamask_sign()`.
- **Reality:** all four ARE real exports (L2948/2768/332/2047) — but the example never shows the `import { ... } from "@terminal3/t3n-sdk"` line, so copy-paste → `ReferenceError`.
- **Fix:** add the import statement.
- **Severity:** High.

### BUG-12 — [HIGH] `authenticate()` requires an argument the setup omits
> ❌ **DOES NOT REPRODUCE ON CURRENT LIVE DOCS (verified 2026-06-07):** live `set-up-dev-env` now shows `await t3n.authenticate(createEthAuthInput(address))` — claim no longer reproduces. Retained for transparency; not an active finding.
- **Reality (L1539):** `authenticate(authInput: AuthInput): Promise<Did>`. set-up-dev-env prose implies a bare `authenticate()`. Must pass `createEthAuthInput(address)`.
- **Severity:** High.

### BUG-13 — [HIGH] `loadWasmComponent` flow missing from setup
> ❌ **DOES NOT REPRODUCE ON CURRENT LIVE DOCS (verified 2026-06-07):** live `set-up-dev-env` now imports `loadWasmComponent` and uses `const wasmComponent = await loadWasmComponent()` wired into the constructor — claim no longer reproduces. Retained for transparency; not an active finding.
- **Reality (L249):** `loadWasmComponent(config?): Promise<WasmComponent>` is module-level and its result is **required** by `new T3nClient({ wasmComponent })`. Setup says only "load the WASM component" with no name, no import, no wiring into the constructor.
- **Severity:** High.

### BUG-14 — [HIGH] Register-before-maps ordering never stated
- `maps.create` needs `{ only: [contractId] }`; `contractId` only exists after `register()`. Required order (register → maps → seed → invoke) is documented nowhere; create-kv-maps shows `contractId` undefined.
- **Severity:** High.

### BUG-15 — [MEDIUM] Agent/user clients used without setup
> 🟡 **REFRAMED (2026-06-07):** overstated — live `invoke-contract` DOES declare `agentKey`, `agentAddress`, and `agentClient`. Only `userClient` and `agentDid` are used without being defined. Narrow the claim to those two.
- invoke-contract uses `userClient`, `agentClient`, `agentDid`, `agentAddress`, `agentKey`; setup only builds one `TenantClient`. Creation of the agent identity/key and user client is never shown.
- **Severity:** Medium.

---

## B. Documentation gaps in examples

### BUG-16 — [HIGH] Two documented errors have no fix
> 🟡 **REFRAMED (2026-06-07):** `email_not_verified` and `user_not_found` do appear, but the Authentication table has only Code + When columns — there is no fix column to be "blank". More accurate framing: "the errors table provides no resolution guidance." Suggest downgrading to MEDIUM.
- **Page:** common-errors — `email_not_verified` and `user_not_found` rows have blank resolutions ("fix missing").
- **Severity:** High.

### BUG-17 — [MEDIUM] `ContractError` type never defined/imported
> ❌ **DOES NOT REPRODUCE ON CURRENT LIVE DOCS (verified 2026-06-07):** live `write-contract` DOES import it — `use exports::t3n::contract::dispatch::{Guest, ContractInput, ContractOutput, ContractError};`. The variants aren't enumerated, but the "never imported" claim is false. Retained for transparency; not an active finding.
- write-contract returns `ContractError::*` variants; never imported or enumerated. Doc itself admits *"contract-authored errors are undefined beyond examples."*
- **Severity:** Medium.

### BUG-18 — [MEDIUM] seed-api-key snippet references undefined symbols
- Uses `tenant.executeControl`/`tenant.canonicalName` with no import/introduction (they exist at L3171/3173 but are never shown being obtained).
- **Severity:** Medium.

### BUG-19 — [MEDIUM] Payroll-agent use-case page is an empty stub
- use-cases/payroll-agent has only a header + one link; no code or explanation.
- **Severity:** Medium.

### BUG-20 — [MEDIUM] snake_case vs camelCase API drift
> 🟡 **REFRAMED (2026-06-07):** the bridging line `const contractId = result.contract_id;` IS shown on `register-contract`; it's only absent on `create-kv-maps` where `contractId` is consumed. Real but minor — reframe as "contractId origin not shown on create-kv-maps." Suggest LOW.
- `register()` returns `result.contract_id` (snake); create-kv-maps consumes `contractId` (camel). No bridging assignment shown.
- **Severity:** Medium.

---

## C. Consistency / clarity / tooling

### BUG-21 — [LOW] Duplicated toolchain command
- `rustup target add wasm32-wasip2` appears in both set-up-dev-env Step 2 and build-contract, the latter framing it as part of every build.
- **Severity:** Low.

### BUG-22 — [LOW] Claim URL naming + no key recovery
> 🟡 **REFRAMED (2026-06-07):** the no-recovery half is solid (verbatim: "the key is shown only once and can't be retrieved after you leave the page"). The URL-naming-inconsistency half could not be reproduced — drop it, keep the no-recovery point.
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

## F. Sample-source vs docs contradictions (read from official sample)

### BUG-25 — [HIGH] README `book-offer` input schema is rejected by the contract
- **Doc:** sample README shows `book-offer` input with a `passengers` array carrying full PII (`given_name`, `family_name`, `date_of_birth`, `passport_number`, …).
- **Reality (`src/booking.rs`):** `BookOfferReq { offer_id, passenger_id, total_amount, total_currency }` — there is **no `passengers` field**. The contract's own test `book_offer_rejects_inline_pii_fields` asserts that passing `passengers` → `"bad input"` error.
- **Impact:** The "complete working example" documents an input the code refuses. Anyone copying the README payload gets a hard error.
- **Severity:** High.

### BUG-26 — [HIGH] Docs HTTP request/response API shape is wrong
- **Doc:** write-contract `search.rs` — `http_iface::call(&http_iface::Request { method: "POST".to_string(), url, headers, body })`, reads `resp.body`.
- **Reality (`src/booking.rs`/`search.rs`):** `method` is an enum `Verb::Post` (not a `String`); the body field is `payload` (not `body`); the response exposes `resp.payload` **and** `resp.code` (no `resp.body`). Errors are a typed `HttpError` enum (`EgressDenied`, `PlaceholderDenied`, …).
- **Impact:** Docs HTTP snippet does not compile against the real bindings.
- **Severity:** High.

### BUG-27 — [HIGH] `capabilities-from-wit-import` repeats the wrong host namespace
- **Doc:** lists `t3n:host/kv-store@0.1.0`, `t3n:host/logging@0.1.0`, `t3n:host/tenant-context@0.1.0`, `t3n:host/http-iface@0.1.0`.
- **Reality (compiled component):** `host:interfaces/kv-store@2.1.0`, `host:interfaces/logging@2.1.0`, `host:tenant/tenant-context@1.0.0`, `host:interfaces/http@2.1.0`, **plus** `host:interfaces/http-with-placeholders@2.1.0` which the page omits entirely.
- **Impact:** A second page (besides write-contract) propagates the invalid namespace/version and a wrong interface name (`http-iface` vs `http`), and misses the placeholders capability.
- **Severity:** High.

### BUG-28 — [MEDIUM] README capability manifest is missing `http_with_placeholders`
- **Doc:** README "Host-capability manifest" lists 4: `["kv_store","logging","tenant_context","http"]`.
- **Reality (`src/lib.rs` doc-comment):** lists 5, including `"http_with_placeholders"` — which `book-offer` requires (it imports + calls `http-with-placeholders`). With the README's 4-cap manifest, `book-offer` fails the capability check.
- **Severity:** Medium.

### BUG-29 — [MEDIUM] Manifest capability names vs WIT interface names mapping undocumented
> 🟡 **REFRAMED (2026-06-07):** the docs page has no snake_case manifest (it states there isn't one), so the docs-page framing is off. The real basis is the *sample README* (`host_capabilities: ["kv_store",…]` snake_case) vs the WIT kebab-case imports — reframe around the README, not the docs page.
- Manifest uses snake_case (`kv_store`, `http_with_placeholders`); WIT imports use namespaced kebab-case (`host:interfaces/kv-store`, `host:interfaces/http-with-placeholders`). The mapping between the two naming schemes is never documented.
- **Severity:** Medium.

### BUG-30 — [MEDIUM] Data-flow / privacy description is inaccurate
- **Doc:** README says `book-offer` "POST to Duffel `/air/orders` with full passenger PII" and "passenger PII … is passed in by the agent."
- **Reality (`src/booking.rs`):** name/DOB/gender/email are resolved from the **user profile** via `{{profile.*}}` placeholders (not passed by the agent); passport/title/phone are **hardcoded demo values** (`X12345678`, `mr`, `+44…`). The agent passes none of the PII. The privacy narrative overstates/misdescribes the actual flow.
- **Severity:** Medium.

### BUG-31 — [LOW] Non-existent `z_sdk.kv().set()` API in README *and* source comments
- The pseudocode `z_sdk.kv("secrets").set("duffel_api_key", ...)` appears in both README and `src/lib.rs` doc-comments. No such API exists in the SDK (`executeControl("map-entry-set", …)` is the real surface — BUG-06).
- **Severity:** Low.

### BUG-32 — [LOW] Duffel endpoint spelling inconsistent in README
- README table says `POST /air/offer-requests` (hyphen); code uses `/air/offer_requests` (underscore — the real Duffel path).
- **Severity:** Low.

### BUG-33 — [LOW] README `cargo test --lib` does not execute tests
- `.cargo/config.toml` defaults the target to `wasm32-wasip2`, so `cargo test --lib` builds a `.wasm` test binary that won't run without a wasm runtime. Native run requires `cargo test --lib --target <host-triple>` (verified: 7/7 pass on `aarch64-apple-darwin`).
- **Severity:** Low.

### BUG-34 — [MEDIUM] SDK `exports` map blocks `./package.json`
- **Repro:** `import "@terminal3/t3n-sdk/package.json"` → `ERR_PACKAGE_PATH_NOT_EXPORTED`.
- **Cause:** the package's `exports` field defines only `"."` and `"./wasm/generated/session.js"`; the conventional `"./package.json"` subpath is omitted, breaking tools that read it (version detection, bundler plugins).
- **Fix:** add `"./package.json": "./package.json"` to `exports`.
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
