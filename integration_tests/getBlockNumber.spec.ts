import { SDK } from "../";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK(SERVER_URL);

test("getBestBlockNumber", async () => {
    const blockNumber = await sdk.getBestBlockNumber();
    expect(typeof blockNumber).toBe("number");
});
