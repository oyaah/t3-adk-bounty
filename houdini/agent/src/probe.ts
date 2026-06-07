// Live probe: real T3 handshake + Eth authenticate. Prints DID + address.
// Never prints the key. Run: set -a; . ./.env; set +a; tsx src/probe.ts
import { connectLive } from "./t3.js";

const r = await connectLive();
if (!r) {
  console.log("harness mode (no AGENT_KEY)");
  process.exit(0);
}
console.log("LIVE auth OK");
console.log("  address:", r.address);
console.log("  did:    ", r.did);
