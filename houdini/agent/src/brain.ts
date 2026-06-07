// The agent's brain: an LLM (OpenAI) turns a natural-language instruction into a
// structured action proposal. Crucially, the brain is UNTRUSTED — it can be
// wrong, jailbroken, or prompt-injected. Whatever it proposes still has to pass
// the TEE guard. "The brain can be tricked; the cage can't."

import OpenAI from "openai";

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

/**
 * Ask the LLM to plan an action from a natural-language instruction. The result
 * is whatever the (untrusted) model decides — it is NOT validated here; the TEE
 * guard is the only authority. Throws if no OPENAI_API_KEY is set.
 */
export async function planAction(instruction: string, nonce: number): Promise<ProposedAction> {
  if (!brainAvailable()) throw new Error("OPENAI_API_KEY not set — add it to .env to enable the LLM brain");
  const client = new OpenAI();
  const res = await client.chat.completions.create({
    model: MODEL,
    response_format: { type: "json_object" },
    messages: [
      { role: "system", content: SYSTEM },
      { role: "user", content: `Suggested nonce: ${nonce}.\nInstruction: ${instruction}` },
    ],
  });
  const raw = res.choices[0]?.message?.content ?? "{}";
  const parsed = JSON.parse(raw) as ProposedAction;
  return {
    action: String(parsed.action ?? "pay_vendor"),
    amount: Number(parsed.amount ?? 0),
    nonce: Number(parsed.nonce ?? nonce),
    rationale: parsed.rationale,
  };
}
