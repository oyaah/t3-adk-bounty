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
export async function connect(): Promise<ConnectResult> {
  const key = process.env.AGENT_KEY;
  const address = process.env.AGENT_ADDRESS;

  if (!key || !address) {
    return { mode: "harness", did: "did:t3n:harness-local" };
  }

  const sdk = await import("@terminal3/t3n-sdk");
  sdk.setEnvironment("testnet"); // module-level function, not client.setEnvironment (BUG-02)
  const wasmComponent = await sdk.loadWasmComponent();
  const client = new sdk.T3nClient({
    wasmComponent, // REQUIRED; there is no apiKey field (BUG-01)
    handlers: { EthSign: sdk.metamask_sign(address, undefined, key) },
  });
  await client.handshake();
  const did = await client.authenticate(sdk.createEthAuthInput(address)); // arg required (BUG-12)
  return { mode: "live", did };
}
