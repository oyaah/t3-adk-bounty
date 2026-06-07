# Corrected ADK Onboarding (what the docs *should* say)

This is a working, end-to-end rewrite of the getting-started flow, derived from the
**shipped SDK** (`@terminal3/t3n-sdk@3.5.0`) and the **official sample**
(`z-tenant-flight`). Every step here is consistent with code that actually compiles
and exports. Each ⚠️ marks where it diverges from the current docs (with the bug id).

## 0. Prerequisites

```bash
# Rust → WASM
rustup target add wasm32-wasip2
cargo install wasm-tools          # for inspecting components (optional but recommended)

# SDK (Node >= 18)
npm install @terminal3/t3n-sdk    # ⚠️ ships 3 moderate vulns via ethers→ws (BUG-24)
```

Claim your DID + developer key + test tokens from the claim page. The key is an
Ethereum private key used to **sign** auth challenges — it is NOT passed to the
client constructor. ⚠️ (BUG-01: docs say "initialize T3nClient with your API key".)

## 1. Configure the SDK (the part docs leave as prose — BUG-05/13/01/02/12)

```js
import {
  T3nClient,
  setEnvironment,          // ⚠️ module-level function, NOT client.setEnvironment (BUG-02)
  loadWasmComponent,       // ⚠️ real loader name; docs only say "load the WASM component" (BUG-13)
  createEthAuthInput,
  metamask_sign,
} from "@terminal3/t3n-sdk";

setEnvironment("testnet");                       // "testnet" | "production"

const wasmComponent = await loadWasmComponent(); // required by the constructor
const address = "0x...";                         // your wallet address
const privateKey = process.env.T3_PRIV_KEY;      // never hardcode

const client = new T3nClient({
  wasmComponent,                                 // ⚠️ REQUIRED. No `apiKey` field exists (BUG-01)
  handlers: { EthSign: metamask_sign(address, undefined, privateKey) },
});

await client.handshake();
const tenantDid = await client.authenticate(createEthAuthInput(address)); // ⚠️ arg required (BUG-12)
const tenantId = tenantDid.slice("did:t3n:".length);
```

## 2. Write the contract (correct WIT — BUG-03/05/26/27)

`wit/world.wit` — use the **real** host namespace/versions:

```wit
package your:contract@0.1.0;
world contract {
  import host:tenant/tenant-context@1.0.0;
  import host:interfaces/logging@2.1.0;
  import host:interfaces/kv-store@2.1.0;
  import host:interfaces/http@2.1.0;                    // search (no PII)
  import host:interfaces/http-with-placeholders@2.1.0;  // booking (PII via placeholders)
  export contracts;                                     // ⚠️ NOT `t3n:contract/dispatch` (BUG-03)
}
```

In Rust, implement the generated `contracts::Guest` trait with one function per
contract method (e.g. `search_offers`, `book_offer`) taking a `GenericInput`
envelope — there is **no** `dispatch`/`ContractInput`/`ContractOutput` (BUG-05).
HTTP uses a `Verb` enum and a `payload` field; responses have `.payload` and
`.code`, and errors are a typed `HttpError` enum (BUG-26).

`Cargo.toml` must use `default-features = false` on deps for the `no_std`/alloc
WASM build (BUG-08).

## 3. Build

```bash
cargo build --target wasm32-wasip2 --release
wasm-tools component wit target/wasm32-wasip2/release/your_contract.wasm
# confirm: exports `...:contracts@...`, imports `host:interfaces/*@2.1.0`
```

## 4. Register the contract

```js
import { readFile } from "fs/promises";
const wasm = await readFile("target/wasm32-wasip2/release/your_contract.wasm");
const reg = await tenant.contracts.register({ tail: "travel/contracts", version: "0.1.0", wasm });
const contractId = reg.contract_id;   // ⚠️ snake_case in the response (BUG-20)
```

## 5. Create the KV map — MISSING from current walkthrough (BUG-04/14/23)

```js
await tenant.maps.create({
  tail: "secrets",
  visibility: "private",
  writers: { only: [contractId] },
  readers: { only: [contractId] },    // ⚠️ MUST be set — governor defaults to deny (BUG-23)
});
```

Order matters: **register → create map → seed secret → invoke** (BUG-14).

## 6. Seed the secret — MISSING from current walkthrough (BUG-04/06)

```js
await tenant.executeControl("map-entry-set", {     // ⚠️ NOT z_sdk.kv().set() (BUG-06/31)
  map_name: tenant.canonicalName("secrets"),
  key: "duffel_api_key",
  value: process.env.DUFFEL_API_KEY,
});
```

## 7. Declare host capabilities — MISSING from walkthrough (BUG-09/28/29)

The capability manifest must include **every** imported interface, mapping the WIT
interface name to its snake_case capability name:

```json
{ "host_capabilities": ["kv_store","logging","tenant_context","http","http_with_placeholders"] }
```
⚠️ The README's 4-cap list omits `http_with_placeholders`, which `book-offer` needs (BUG-28).

## 8. Invoke (with import lines docs omit — BUG-11)

```js
import { getNodeUrl, getScriptVersion } from "@terminal3/t3n-sdk";  // ⚠️ docs never import these (BUG-11)
const SCRIPT = `z:${tenantId}:travel/contracts`;
const version = await getScriptVersion(getNodeUrl(), SCRIPT);
const result = await agentClient.executeAndDecode({
  script_name: SCRIPT, script_version: version,
  function_name: "search-offers",
  input: { origin: "LHR", destination: "JFK", departure_date: "2026-07-15",
           cabin_class: "economy", adult_count: 1 },   // ⚠️ matches real SearchOffersReq (BUG-25)
});
```
