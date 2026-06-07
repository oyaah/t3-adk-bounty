// Houdini public red-team server. Anyone can POST an instruction; an LLM turns
// it into an action; the action is judged by the real Houdini TEE contract on
// Terminal 3 testnet (or a local enclave-faithful guard when no live key is
// configured). The LLM is untrusted — jailbreak it all you want; the mandate is
// enforced inside the enclave, not here.
import express from "express";
import cors from "cors";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";
import { connectLive, type LiveSession } from "./t3.js";
import { planAction, brainAvailable } from "./brain.js";
import { newOwner, signMandate, type SignedMandate } from "./mandate.js";
import { callContract } from "./agent.js";

process.stdout.write("[houdini] booting…\n");
const __dir = dirname(fileURLToPath(import.meta.url));
const PORT = Number(process.env.PORT ?? 8787);
const NOW = () => Math.floor(Date.now() / 1000);

const MANDATE_FIELDS = {
  mandate_id: "public-treasury",
  per_tx_cap: 300,
  budget_total: 500,
  allowed_actions: ["pay_vendor"],
};

// One owner + signed mandate for the public demo, regenerated per process start.
const owner = newOwner();
let signed: SignedMandate = signMandate({ ...MANDATE_FIELDS, expiry_unix: NOW() + 365 * 86400 }, owner);

// Live session (lazy). If AGENT_KEY is set we register + use the real TEE; else
// we fall back to the local enclave-faithful guard binary (still the real Rust
// enforcement, just not remote).
let live: LiveSession | null = null;
let scriptName = "";
let scriptVersion = "";
let mode: "live-tee" | "local-guard" = "local-guard";

async function initLive() {
  if (!process.env.AGENT_KEY) return;
  const s = await connectLive();
  if (!s) return;
  live = s;
  const tid = (typeof s.did === "string" ? s.did : (s.did as any).value).replace(/^did:t3n:/, "");
  scriptName = `z:${tid}:houdini-guard`;
  scriptVersion = await s.sdk.getScriptVersion(s.sdk.getNodeUrl(), scriptName);
  mode = "live-tee";
  console.log(`[houdini] LIVE TEE mode: ${scriptName}@${scriptVersion}`);
}

async function judge(proposal: { action: string; amount: number; nonce: number }) {
  const payload = { signed, request: proposal, now_unix: NOW() };
  if (mode === "live-tee" && live) {
    const res: any = await (live.client as any).executeAndDecode({
      script_name: scriptName,
      script_version: scriptVersion,
      function_name: "attack",
      input: payload,
    });
    return res as { allowed: boolean; reason: string | null; spent: number };
  }
  // Local enclave-faithful guard (the same Rust logic, via the eval binary).
  return callContract({ signed, request: proposal, now_unix: NOW() });
}

const app = express();
app.use(cors());
app.use(express.json());
app.use(express.static(resolve(__dir, "../../web")));

app.get("/api/mandate", (_req, res) => {
  res.json({
    mode,
    mandate: { ...MANDATE_FIELDS },
    owner_pub: owner.pubHex,
    brain: brainAvailable() ? "openai" : "heuristic",
    script: scriptName || null,
  });
});

// The attack endpoint: free-text instruction -> LLM proposal -> TEE verdict.
app.post("/api/attack", async (req, res) => {
  try {
    const instruction = String(req.body?.instruction ?? "").slice(0, 500);
    if (!instruction) return res.status(400).json({ error: "instruction required" });
    const nonce = Date.now() % 2_000_000_000;
    const proposal = await planAction(instruction, nonce);
    const verdict = await judge(proposal);
    res.json({ instruction, proposal, verdict, mode });
  } catch (e) {
    res.status(500).json({ error: (e as Error).message });
  }
});

app.listen(PORT, async () => {
  console.log(`[houdini] listening on :${PORT} (brain=${brainAvailable() ? "openai" : "none"})`);
  await initLive().catch((e) => console.log(`[houdini] live init skipped: ${e.message}`));
});
