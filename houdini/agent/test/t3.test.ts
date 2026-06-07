import { describe, it, expect } from "vitest";
import { connect, connectLive } from "../src/t3.js";

describe("T3 auth", () => {
  it("falls back to honest harness mode when no agent key is set", async () => {
    const saved = process.env.AGENT_KEY;
    delete process.env.AGENT_KEY;
    const r = await connect();
    expect(r.mode).toBe("harness");
    expect(r.did).toMatch(/^did:t3n:/);
    if (saved !== undefined) process.env.AGENT_KEY = saved;
  });

  it("connectLive returns null without a key (no silent half-init)", async () => {
    const saved = process.env.AGENT_KEY;
    delete process.env.AGENT_KEY;
    expect(await connectLive()).toBeNull();
    if (saved !== undefined) process.env.AGENT_KEY = saved;
  });
});
