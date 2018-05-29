import { SDK, H256 } from "../";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK(SERVER_URL);

test("getBlockHash - latest", async () => {
    const hash = await sdk.getBlockHash(await sdk.getBlockNumber());
    expect(hash instanceof H256).toBeTruthy();
});

test("getBlockHash - latest + 1", async () => {
    const hash = await sdk.getBlockHash(await sdk.getBlockNumber() + 1);
    expect(hash).toBe(null);
});
