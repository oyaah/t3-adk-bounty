# PROOF.md — captured command outputs

All findings below are reproducible on a clean machine. Generated 2026-06-07T08:09Z.

## Toolchain
```
node v25.9.0 | npm 11.12.1
rustc 1.96.0 (ac68faa20 2026-05-25)
wasm-tools 1.251.0
wasm target: wasm32-wasip2
@terminal3/t3n-sdk: 3.5.0
```

## 1. Official sample builds clean (control)
```
$ cargo build --target wasm32-wasip2 --release
Jun z-tenant-flight/target/wasm32-wasip2/release/z_tenant_flight.wasm
```

## 2. Sample tests + clippy pass (control)
```
$ cargo test --lib --target aarch64-apple-darwin
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
$ cargo clippy --all-targets -- -D warnings  => exit 0 (clean)
```

## 3. Compiled component WIT (disproves docs BUG-03 / BUG-27)
```
$ wasm-tools component wit .../z_tenant_flight.wasm
package root:component;

world root {
  import host:tenant/tenant-context@1.0.0;
  import host:interfaces/logging@2.1.0;
  import host:interfaces/kv-store@2.1.0;
  import host:interfaces/http@2.1.0;
  import host:interfaces/http-with-placeholders@2.1.0;
  import wasi:io/poll@0.2.6;
```

## 4. SDK-surface proof harness (BUG-01/02/06/11/12/13)
```
$ node verify.mjs
# Verifying against @terminal3/t3n-sdk@3.5.0

✅ BUG-02: setEnvironment is a MODULE export (not a T3nClient method)
✅ BUG-01: T3nClientConfig has NO apiKey field (requires wasmComponent)
✅ BUG-01b: new T3nClient({ apiKey }) throws (wasmComponent missing)
✅ BUG-11: helper 'getNodeUrl' is a real named export (docs use it without importing)
✅ BUG-11: helper 'getScriptVersion' is a real named export (docs use it without importing)
✅ BUG-11: helper 'createEthAuthInput' is a real named export (docs use it without importing)
✅ BUG-11: helper 'metamask_sign' is a real named export (docs use it without importing)
✅ BUG-12: authenticate(authInput) declared with a required argument
✅ BUG-13: loadWasmComponent is a module-level export
✅ BUG-06: executeControl + canonicalName exist on TenantClient (kv().set() does not)

# 10 bugs confirmed present, 0 not reproduced
```

> ⚠️ **Re-verification note (2026-06-07):** this harness asserts the SDK's **shape** (no `apiKey` field, `setEnvironment` is module-level, helpers are real exports). It does NOT check what the docs currently say. Against the **current live docs**, BUG-01/02/11/12/13 **no longer reproduce** — the docs were corrected to match the SDK. Still valid here: **BUG-06** (`kv().set()` does not exist; `executeControl`/`canonicalName` are the real surface). Full per-bug status in BUGLOG.md.

## 5. npm audit (BUG-24)
```
# npm audit report

ws  8.0.0 - 8.20.0
Severity: moderate
ws: Uninitialized memory disclosure - https://github.com/advisories/GHSA-58qx-3vcg-4xpx
No fix available
node_modules/ws
  ethers  >=6.0.0-beta.1
  Depends on vulnerable versions of ws
  node_modules/ethers
    @terminal3/t3n-sdk  *
    Depends on vulnerable versions of ethers
```
