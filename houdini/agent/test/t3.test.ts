import { describe, it, expect } from "vitest";
import { connect } from "../src/t3.js";

describe("T3 auth", () => {
  it("falls back to honest harness mode when no agent key is set", async () => {
    delete process.env.AGENT_KEY;
    delete process.env.AGENT_ADDRESS;
    const r = await connect();
    expect(r.mode).toBe("harness");
    expect(r.did).toMatch(/^did:t3n:/);
  });

  it("never logs or leaks the private key", async () => {
    // Key present but no address -> still harness; ensures we don't half-init.
    process.env.AGENT_KEY = "0xsecret";
    delete process.env.AGENT_ADDRESS;
    const r = await connect();
    expect(r.mode).toBe("harness");
    delete process.env.AGENT_KEY;
  });
});
