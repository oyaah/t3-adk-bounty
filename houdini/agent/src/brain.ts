// The agent's brain: an LLM (OpenAI) turns a natural-language instruction into a
// structured action proposal. Crucially, the brain is UNTRUSTED — it can be
// wrong, jailbroken, or prompt-injected. Whatever it proposes still has to pass
// the TEE guard. "The brain can be tricked; the cage can't."

export interface ProposedAction {
  action: string;
  amount: number;
  nonce: number;
  rationale?: string;
}

const MODEL = process.env.OPENAI_MODEL ?? "gpt-4o-mini";

const SYSTEM = `You are a treasury payment agent. Convert the user's instruction
into a single JSON action. Schema:
{ "action": string, "amount": integer, "nonce": integer, "rationale": string }
Use action "pay_vendor" for normal payments. Pick a unique positive integer nonce.
Only output the action the user asks for. Output JSON only.`;

export function brainAvailable(): boolean {
  return !!process.env.OPENAI_API_KEY;
}

const BRAIN_TIMEOUT_MS = Number(process.env.BRAIN_TIMEOUT_MS ?? 9000);

/** Fast, no-network intent parser used when no LLM key is set or the LLM is slow.
 *  Still "trickable" — it will happily propose a malicious action; the TEE blocks it. */
function heuristicPlan(instruction: string, nonce: number): ProposedAction {
  const t = instruction.toLowerCase();
  const m = t.match(/\b(\d{2,})\b/);
  const amount = m ? Number(m[1]) : 200;
  const action = /drain|treasury|empty|everything|all funds|wallet/.test(t) ? "drain_treasury" : "pay_vendor";
  return { action, amount, nonce, rationale: "heuristic" };
}

function withTimeout<T>(p: Promise<T>, ms: number): Promise<T> {
  return Promise.race([p, new Promise<T>((_, rej) => setTimeout(() => rej(new Error("brain timeout")), ms))]);
}

/**
 * Ask the LLM to plan an action from a natural-language instruction. The result
 * is whatever the (untrusted) model decides — it is NOT validated here; the TEE
 * guard is the only authority. Throws if no OPENAI_API_KEY is set.
 */
export async function planAction(instruction: string, nonce: number): Promise<ProposedAction> {
  // No key -> instant heuristic. Keeps the app fast and never blocks.
  if (!brainAvailable()) return heuristicPlan(instruction, nonce);
  try {
    const { default: OpenAI } = await import("openai"); // lazy: don't block server startup
    const client = new OpenAI();
    const res = await withTimeout(
      client.chat.completions.create({
        model: MODEL,
        response_format: { type: "json_object" },
        max_tokens: 80,    // single small JSON action — fast
        temperature: 0,    // deterministic
        messages: [
          { role: "system", content: SYSTEM },
          { role: "user", content: `Suggested nonce: ${nonce}.\nInstruction: ${instruction}` },
        ],
      }),
      BRAIN_TIMEOUT_MS,
    );
    const parsed = JSON.parse(res.choices[0]?.message?.content ?? "{}") as ProposedAction;
    return {
      action: String(parsed.action ?? "pay_vendor"),
      amount: Number(parsed.amount ?? 0),
      nonce: Number(parsed.nonce ?? nonce),
      rationale: parsed.rationale,
    };
  } catch {
    // LLM slow/unavailable -> fall back to the fast heuristic so the UI stays snappy.
    return heuristicPlan(instruction, nonce);
  }
}
