# Deploy Houdini so anyone can attack it

Two pieces: the **agent backend** (LLM brain + Terminal 3 TEE) on **Railway**, and the **static frontend** on **Vercel**.

## 1. Backend → Railway

The backend (`houdini/agent`) exposes `POST /api/attack` and `GET /api/mandate`. In live mode it calls the already-registered Houdini contract (`contract_id 19`) in the Terminal 3 TEE.

```bash
cd houdini/agent
# Option A: Railway CLI
railway init            # or: railway link  (to an existing project)
railway up              # uses the Dockerfile
# Option B: Railway dashboard → New Project → Deploy from GitHub repo,
#           root directory = houdini/agent
```

**Set these env vars in Railway** (Service → Variables):
- `OPENAI_API_KEY` — the agent's LLM brain (without it, a fast heuristic planner is used).
- `AGENT_KEY` — your Terminal 3 developer key (the `0x…` Ed25519/eth key). Enables **live TEE** mode. Without it the backend uses the local enclave-faithful guard (needs the `eval` binary; live mode does not).
- `PORT` — Railway sets this automatically; the server reads it.

Copy the resulting URL, e.g. `https://houdini-agent.up.railway.app`.

> Security: the keys live only in Railway's env. They are never in the repo (`.env` is gitignored).

## 2. Frontend → Vercel

```bash
cd houdini/web
# Set the backend URL the UI calls:
#   edit config.js -> window.HOUDINI_API = "https://houdini-agent.up.railway.app"
vercel            # or: import the repo in the Vercel dashboard, root = houdini/web
```

Vercel serves `houdini/web/` as a static site. The page calls the Railway backend via `window.HOUDINI_API` (set in `config.js`). Enable CORS is already handled by the backend (`cors()`).

## 3. Try it

Open the Vercel URL. Type any instruction — legit or a jailbreak — and watch the LLM's proposal hit the enclave: ALLOW (green) or BLOCKED (red, with the enforced reason).

## Run locally (no hosting)

```bash
cd houdini/agent
cp .env.example .env     # add OPENAI_API_KEY (+ AGENT_KEY for live TEE)
cargo build --release --bin eval --manifest-path ../contract/Cargo.toml
npx tsx src/server.ts    # http://localhost:8787 (frontend served at /)
```
