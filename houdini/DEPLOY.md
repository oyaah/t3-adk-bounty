# Deploy Houdini ($0) — Vercel + Render free

Frontend (static) → **Vercel (Hobby, free)**. Backend (agent + TEE) → **Render free web service**.
Total cost: $0. (Render free sleeps after ~15 min idle; first hit cold-starts ~30–60s.)

## 1. Backend → Render (free)

The repo has [`houdini/agent/render.yaml`](agent/render.yaml) (a Render Blueprint).

1. Render dashboard → **New → Blueprint** → connect this GitHub repo. It picks up `render.yaml` (service `houdini-agent`, Docker, **plan: free**).
2. Set these as **Secret** env vars (Service → Environment):
   - `AGENT_KEY` — a **dedicated BURNER Terminal 3 testnet key** (Ethereum private key). **Never your main key.** If Render leaks it, blast radius ≈ 0. Without it the backend runs the local enclave-faithful guard.
   - `OPENAI_API_KEY` — the LLM brain. Without it a fast heuristic planner is used (still trickable, still blocked).
   - `ALLOWED_ORIGIN` — your Vercel URL (e.g. `https://houdini.vercel.app`); locks CORS. Comma-separate for multiple.
   - `RATE_LIMIT_PER_MIN` — optional (default 12). Per-IP cap on `/api/attack`.
3. Deploy → copy the URL, e.g. `https://houdini-agent.onrender.com`.

> Secrets live only in Render's encrypted env — never in the repo (`.env` is gitignored). For rotation, point them at Doppler/Infisical later.

## 2. Frontend → Vercel (free)

```bash
cd houdini/web
# point the UI at your Render backend:
#   edit config.js -> window.HOUDINI_API = "https://houdini-agent.onrender.com"
vercel            # or import the repo in the Vercel dashboard, root = houdini/web
```

Then set `ALLOWED_ORIGIN` on Render to the Vercel URL so CORS matches.

## Security posture (already wired)
- **Per-IP rate limit** on `/api/attack` (`express-rate-limit`, default 12/min) — caps spam / OpenAI cost / T3 load.
- **CORS locked** to `ALLOWED_ORIGIN` in prod (open only when unset, for local dev).
- **Burner key** for `AGENT_KEY` (minimal/zero funds) — the real safeguard.
- JSON body capped at 8kb; instruction capped at 500 chars; small `max_tokens`.

## Run locally (no hosting)
```bash
cd houdini/agent
cp .env.example .env     # add OPENAI_API_KEY (+ AGENT_KEY for live TEE)
cargo build --release --bin eval --manifest-path ../contract/Cargo.toml
npm start                # http://localhost:8787
```
