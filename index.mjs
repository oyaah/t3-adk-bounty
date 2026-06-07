// T3 ADK onboarding driver — stitched from docs.terminal3.io walkthrough + tips pages.
// Purpose: run the FULL documented flow end-to-end to surface runtime bugs.
// Many symbols below are taken verbatim from docs that never define/import them
// (see BUGLOG BUG-03/04/05/08). Running this is expected to expose those gaps.
//
// Usage:
//   export T3_API_KEY=0x...        # your developer key
//   export DUFFEL_API_KEY=...      # any test value
//   node index.mjs

import { readFile } from "fs/promises";
// NOTE: docs never show which symbols the SDK actually exports. Guessing from examples:
import {
  T3nClient,
  TenantClient,
  getNodeUrl,          // used in invoke-contract, never imported in docs  -> BUG-03
  getScriptVersion,    // same
  createEthAuthInput,  // same
  metamask_sign,       // same
} from "@terminal3/t3n-sdk";

const T3_API_KEY = process.env.T3_API_KEY;
const DUFFEL_API_KEY = process.env.DUFFEL_API_KEY ?? "test-key";
const WASM_PATH = "z-tenant-flight/target/wasm32-wasip2/release/z_tenant_flight.wasm";
const CONTRACT_TAIL = "travel/contracts";
const CONTRACT_VERSION = "0.1.0";

async function main() {
  // --- Step 4 (setup): configure SDK. Docs give PROSE only -> BUG-05.
  const client = new T3nClient({ apiKey: T3_API_KEY });
  await client.setEnvironment("testnet");
  // docs: "load the WASM component" — method name never given. Guessing:
  const wasmComponent = await client.loadWasm?.();
  const tenantDid = await client.authenticate();      // args unknown -> BUG-05
  const tenantId = tenantDid.slice("did:t3n:".length);
  const tenant = new TenantClient({ did: tenantDid, nodeUrl: getNodeUrl() });

  // --- Register contract
  const wasmBytes = await readFile(WASM_PATH);
  const reg = await tenant.contracts.register({
    tail: CONTRACT_TAIL,
    version: CONTRACT_VERSION,
    wasm: wasmBytes,
  });
  const contractId = reg.contract_id;   // snake_case in docs -> BUG-10
  console.log("registered contract_id:", contractId);

  // --- Create KV map (MISSING from walkthrough -> BUG-01/14). readers mandatory.
  try {
    await tenant.maps.create({
      tail: "secrets",
      visibility: "private",
      writers: { only: [contractId] },
      readers: { only: [contractId] },
    });
  } catch (e) {
    if (!/already exists/i.test(String(e))) throw e;   // idempotent per common-errors
  }

  // --- Seed secret (MISSING from walkthrough -> BUG-01/08)
  await tenant.executeControl("map-entry-set", {
    map_name: tenant.canonicalName("secrets"),
    key: "duffel_api_key",
    value: DUFFEL_API_KEY,
  });

  // --- Invoke. Needs agentClient + user egress grant the docs never set up -> BUG-04.
  const TENANT_SCRIPT = `z:${tenantId}:${CONTRACT_TAIL}`;
  const scriptVersion = await getScriptVersion(getNodeUrl(), TENANT_SCRIPT);
  const result = await tenant.executeAndDecode({
    script_name: TENANT_SCRIPT,
    script_version: scriptVersion,
    function_name: "search-offers",
    input: { origin: "LHR", destination: "JFK", departure_date: "2026-07-15" },
  });
  console.log("invoke result:", result);
}

main().catch((e) => {
  console.error("RUNTIME FAILURE (log this into BUGLOG section D):\n", e);
  process.exit(1);
});
