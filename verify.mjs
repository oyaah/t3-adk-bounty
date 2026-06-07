// Runnable proof harness for the documentation bugs.
// Asserts each SDK-surface claim against the SHIPPED @terminal3/t3n-sdk@3.5.0.
// Run: node verify.mjs   (exit 0 = all bugs confirmed present)
import * as sdk from "@terminal3/t3n-sdk";
import { readFileSync } from "fs";

// NOTE: cannot `import "@terminal3/t3n-sdk/package.json"` — the package's exports
// map omits "./package.json" (BUG-34). Read files directly from node_modules.
const base = new URL("./node_modules/@terminal3/t3n-sdk/", import.meta.url);
const sdkPkg = JSON.parse(readFileSync(new URL("package.json", base), "utf8"));
const dts = readFileSync(new URL("dist/index.d.ts", base), "utf8");

let pass = 0, fail = 0;
const check = (id, claim, cond) => {
  const ok = !!cond;
  console.log(`${ok ? "✅" : "❌"} ${id}: ${claim}`);
  ok ? pass++ : fail++;
};

console.log(`# Verifying against @terminal3/t3n-sdk@${sdkPkg.version}\n`);

// BUG-02: docs call client.setEnvironment(); real setEnvironment is a module-level export.
check("BUG-02", "setEnvironment is a MODULE export (not a T3nClient method)",
  typeof sdk.setEnvironment === "function" &&
  typeof sdk.T3nClient?.prototype?.setEnvironment === "undefined");

// BUG-01: docs say `new T3nClient({ apiKey })`; real config has no apiKey, requires wasmComponent.
check("BUG-01", "T3nClientConfig has NO apiKey field (requires wasmComponent)",
  !/T3nClientConfig[\s\S]*?apiKey/.test(dts) &&
  /interface T3nClientConfig\s*\{[\s\S]*?wasmComponent:\s*WasmComponent/.test(dts));

// BUG-01b: constructing with only an apiKey must throw (no wasmComponent).
let threw = false;
try { new sdk.T3nClient({ apiKey: "0xdeadbeef" }); } catch { threw = true; }
check("BUG-01b", "new T3nClient({ apiKey }) throws (wasmComponent missing)", threw);

// BUG-11: invoke walkthrough helpers exist as real exports but docs never import them.
for (const h of ["getNodeUrl", "getScriptVersion", "createEthAuthInput", "metamask_sign"]) {
  check("BUG-11", `helper '${h}' is a real named export (docs use it without importing)`,
    typeof sdk[h] === "function");
}

// BUG-12: authenticate requires an AuthInput argument (docs imply none).
check("BUG-12", "authenticate(authInput) declared with a required argument",
  /authenticate\(authInput:\s*AuthInput\)/.test(dts));

// BUG-13: loadWasmComponent is the real module-level loader (docs say only 'load the WASM component').
check("BUG-13", "loadWasmComponent is a module-level export",
  typeof sdk.loadWasmComponent === "function");

// BUG-06: docs/README seed via z_sdk.kv().set() — no such API; executeControl is the real surface.
check("BUG-06", "executeControl + canonicalName exist on TenantClient (kv().set() does not)",
  /executeControl\(functionName:/.test(dts) && /canonicalName\(tail:/.test(dts) &&
  !/\bkv\(["'`]/.test(dts));

console.log(`\n# ${pass} bugs confirmed present, ${fail} not reproduced`);
process.exit(fail === 0 ? 0 : 1);
