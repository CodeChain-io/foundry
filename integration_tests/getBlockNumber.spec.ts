import { SDK } from "../";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK({ server: SERVER_URL });

test("getBestBlockNumber", async () => {
    const blockNumber = await sdk.rpc.chain.getBestBlockNumber();
    expect(typeof blockNumber).toBe("number");
});
