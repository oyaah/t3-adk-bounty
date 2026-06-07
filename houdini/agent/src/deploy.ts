// LIVE end-to-end: claim the tenant, register the real Houdini WASM contract,
// and invoke it on the Terminal 3 testnet TEE with a real signed mandate and a
// real escape attempt. No mocks. Run:
//   set -a; . ./.env; set +a; npx tsx src/deploy.ts
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";
import { connectLive } from "./t3.js";
import { newOwner, signMandate, type SignedMandate } from "./mandate.js";

const __dir = dirname(fileURLToPath(import.meta.url));
const WASM = resolve(__dir, "../../contract/target/wasm32-wasip2/release/houdini_contract.wasm");
const TAIL = "houdini-guard";
const VERSION = "0.1.0";
const NOW = Math.floor(Date.now() / 1000);

function didString(did: unknown): string {
  if (typeof did === "string") return did;
  if (did && typeof did === "object" && "value" in did) return String((did as any).value);
  return String(did);
}

function payload(signed: SignedMandate, req: object, prior_spent = 0, used_nonces: number[] = []) {
  return { signed, request: req, prior_spent, used_nonces, now_unix: NOW };
}

const G = "\x1b[32m", R = "\x1b[31m", D = "\x1b[2m", B = "\x1b[1m", X = "\x1b[0m";

async function main() {
  const s = await connectLive();
  if (!s) throw new Error("no AGENT_KEY — set it in .env for live deploy");
  const { sdk, client } = s;
  const tenantDid = didString(s.did);
  const tenantId = tenantDid.replace(/^did:t3n:/, "");
  console.log(`\n[1] authenticated  did=${tenantDid}  addr=${s.address}`);

  const baseUrl = sdk.getNodeUrl();
  const tenant = new sdk.TenantClient({
    environment: "testnet",
    t3n: client as any,
    tenantDid,
    baseUrl,
    endpoint: baseUrl,
  });

  // [2] Claim the tenant (idempotent — ignore "already" errors).
  try {
    await tenant.tenant.claim();
    console.log("[2] tenant claimed");
  } catch (e) {
    console.log(`[2] tenant claim: ${(e as Error).message.slice(0, 120)} (continuing)`);
  }

  // [3] Register the real WASM contract.
  const wasm = await readFile(WASM);
  console.log(`[3] registering ${TAIL}@${VERSION} (${wasm.length} bytes wasm)…`);
  try {
    const reg = await tenant.contracts.register({ tail: TAIL, version: VERSION, wasm });
    console.log(`[3] registered:`, JSON.stringify(reg).slice(0, 200));
  } catch (e) {
    const msg = (e as Error).message;
    if (/not higher than current/.test(msg)) {
      console.log(`[3] already registered at ${VERSION} (idempotent re-run)`);
    } else {
      throw e;
    }
  }

  const scriptName = `z:${tenantId}:${TAIL}`;
  const scriptVersion = await sdk.getScriptVersion(sdk.getNodeUrl(), scriptName);
  console.log(`[4] script ${scriptName} @ ${scriptVersion}`);

  // [5] Real signed mandate + a legit action and an escape attempt, on-chain.
  const owner = newOwner();
  const signed = signMandate(
    { mandate_id: "treasury-q3", per_tx_cap: 300, budget_total: 500, allowed_actions: ["pay_vendor"], expiry_unix: NOW + 86_400 },
    owner,
  );

  // Each row: [function, kind, label, payload]. Every verdict below is returned
  // by the REAL remote TEE running our registered contract — not a local mock.
  const forged = (() => { const f = structuredClone(signed); f.mandate.budget_total = 1_000_000; return f; })();
  const rows: Array<[string, "legit" | "escape", string, object]> = [
    ["act", "legit", "pay vendor invoice", payload(signed, { action: "pay_vendor", amount: 200, nonce: 1 })],
    ["act", "legit", "pay SaaS subscription", payload(signed, { action: "pay_vendor", amount: 100, nonce: 2 }, 200)],
    ["attack", "escape", "budget-bust (9999 > cap)", payload(signed, { action: "pay_vendor", amount: 9999, nonce: 3 })],
    ["attack", "escape", "nonce replay (reuse 1)", payload(signed, { action: "pay_vendor", amount: 50, nonce: 1 }, 200, [1])],
    ["attack", "escape", "forged mandate (tampered)", payload(forged, { action: "pay_vendor", amount: 50, nonce: 4 })],
    ["attack", "escape", "scope escalation (drain)", payload(signed, { action: "drain_treasury", amount: 10, nonce: 5 })],
    ["attack", "escape", "PII exfil", payload(signed, { action: "pay_vendor", amount: 10, nonce: 6, returns_pii: true })],
  ];

  console.log(`\n  ${B}HOUDINI — LIVE on Terminal 3 TEE${X}  ${D}contract ${scriptName}${X}\n`);
  let escapes = 0, blocked = 0;
  for (const [fn, kind, label, input] of rows) {
    if (kind === "escape") escapes++;
    const res: any = await (client as any).executeAndDecode({
      script_name: scriptName, script_version: scriptVersion, function_name: fn, input,
    });
    const tag = kind === "legit" ? `${D}legit ${X}` : `${R}ATTACK${X}`;
    if (res.allowed) {
      console.log(`  ${tag}  ${G}✓ ALLOW${X}   ${label.padEnd(28)} ${D}spent=${res.spent}${X}`);
    } else {
      if (kind === "escape") blocked++;
      console.log(`  ${tag}  ${R}✗ BLOCKED${X} ${label.padEnd(28)} ${R}${res.reason}${X}`);
    }
  }
  const ok = blocked === escapes;
  console.log(`\n  ${ok ? G : R}${B}${blocked}/${escapes} escapes blocked by the live TEE${X}\n`);
}

main().catch((e) => {
  console.error("\nLIVE deploy error:", e?.message || e);
  process.exit(1);
});
