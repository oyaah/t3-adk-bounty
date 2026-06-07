// Real Terminal 3 SDK authentication, wired correctly per the verified SDK
// surface (see ../../FIXED-ONBOARDING.md): the API key is NOT passed to the
// client; auth is an EthSign challenge/response. The SDK is dynamically
// imported only in live mode, so harness mode (and CI) need no network and no
// wasm component load.

export type ConnectResult =
  | { mode: "live"; did: string }
  | { mode: "harness"; did: string };

/**
 * Connect to the T3 network. Live when AGENT_KEY + AGENT_ADDRESS are present;
 * otherwise an honestly-labeled local harness identity (KTD-2). The live path
 * uses the correct flow: setEnvironment -> loadWasmComponent -> new T3nClient
 * ({ wasmComponent, handlers: { EthSign } }) -> handshake -> authenticate.
 */
export interface LiveSession {
  mode: "live";
  did: string;
  address: string;
  /** The authenticated T3nClient — pass as `t3n` to a TenantClient. */
  client: unknown;
  sdk: typeof import("@terminal3/t3n-sdk");
}

export async function connect(): Promise<ConnectResult> {
  const s = await connectLive();
  if (s) return { mode: "live", did: s.did };
  return { mode: "harness", did: "did:t3n:harness-local" };
}

/**
 * Full live connection. Returns null (harness) when AGENT_KEY is absent.
 * The address is derived from the key via the SDK — no separate env needed,
 * no hardcoding.
 */
export async function connectLive(): Promise<LiveSession | null> {
  const key = process.env.AGENT_KEY;
  if (!key) return null;

  const sdk = await import("@terminal3/t3n-sdk");
  sdk.setEnvironment("testnet"); // module-level function, not client.setEnvironment (BUG-02)
  const address = sdk.eth_get_address(key); // derive address from the private key
  const wasmComponent = await sdk.loadWasmComponent();
  const client = new sdk.T3nClient({
    wasmComponent, // REQUIRED; there is no apiKey field (BUG-01)
    handlers: { EthSign: sdk.metamask_sign(address, undefined, key) },
  });
  await client.handshake();
  const rawDid = await client.authenticate(sdk.createEthAuthInput(address)); // arg required (BUG-12)
  const did = String((rawDid as any)?.value ?? rawDid);
  return { mode: "live", did, address, client, sdk };
}
