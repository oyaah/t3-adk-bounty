# 🤝 Handoff — Parshiv → Yash

Quick status so you can take it forward. Everything below is on **`master`**, pushed, working tree clean, **CI green**.

---

## ✅ What I changed / verified this session

### $300 Houdini track
- **Verified the whole thing builds + runs on a clean Windows machine** (installed Rust fresh):
  - contract builds host **+ wasm32-wasip2** (~227 KB component)
  - **6/6 escape-matrix tests pass**
  - `cargo run --bin redroom` → **5/5 escapes blocked**
  - agent **6/6 tests pass**
- **Fixed a real cross-platform bug**: `agent/src/agent.ts` `evalBinPath()` didn't append `.exe` on Windows, so the 4 bridge tests were silently **skipping** (and the agent wouldn't run on Windows). Now fixed → all 6 run/pass.
- **Added `houdini/demo/index.html`** — a self-contained, animated visual "Red Room" (no build, no backend). Open it in any browser; click "▶ Run the escape matrix". This is our clickable-UI answer to competitors with deployed front-ends.
- **Added root `README.md`** — judge-facing front door unifying both tracks; leads with Houdini + the "the brain can be tricked, the cage can't" angle.

### $200 bug track — ⚠️ NEEDS YOUR DECISION
- I re-verified all bugs against the **current live docs** and found **7 no longer reproduce** (the docs were corrected since discovery), incl. the old headline criticals **BUG-01 / BUG-02**, plus a few overstated ones.
- **`master`'s `BUGLOG.md` / `SUBMISSION.md` / `PROOF.md` now hold the HONEST re-verified version** (flagged `DOES NOT REPRODUCE ON CURRENT LIVE DOCS`, corrected counts: 34 documented / 27 reproduce).
- **Why:** the judges are the Terminal 3 team — if they click BUG-01/02 on their own live docs and it doesn't reproduce, it hurts us. The honest version protects credibility.
- **Your call:** your `feat/houdini` branch had a **37-bug / "live testnet confirmed"** version with 2 extra live findings. The ideal final = **your live findings + these honesty flags.** Decide which you submit for $200. (My honest re-verification detail is also preserved on the `parshiv` branch.)

---

## ⚠️ Important warnings
- **Do NOT merge the `feat/houdini → master` PR** GitHub is offering — different git lineage; it would **overwrite the honest bug log** on master.
- Everything we want is **already on `master`** directly. `master` is the submission branch.

---

## ▶️ How to run / verify (fresh terminal so `cargo` is on PATH)
```bash
cd houdini/contract
cargo run --bin redroom          # visual matrix: 2 legit ALLOW, 5/5 BLOCKED
cargo test                       # 6/6 escape-matrix tests
cd ../agent && npm test          # 6/6 (incl. TS->Rust bridge)
# Live on the real TEE (needs AGENT_KEY in agent/.env):
npx tsx src/deploy.ts            # registers + runs the matrix on the real enclave
```
No Rust? Just open `houdini/demo/index.html` in a browser.

---

## ⏳ What's LEFT (human-only)
1. **🎥 Record the demo video** — script is in `houdini/DEMO.md` (Parshiv also has a tight 90-sec read-aloud script).
2. **📤 Submit on DoraHacks** — BUIDL #1 = $300 (Parshiv), BUIDL #2 = $200 (Yash).
3. **🤝 Decide the $200 bug-log version** (see above).

---

## 📍 Branch map
- **`master`** — the submission (Houdini + demo + README + honest bug log). ✅ use this.
- **`feat/houdini`** — your original Houdini build + your 37-bug/live BUGLOG (preserved).
- **`parshiv`** — the honest bug-log re-verification (now reflected on master).
