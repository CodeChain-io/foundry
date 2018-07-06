import { SDK } from "../";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK({ server: SERVER_URL });

test("getBlockHash - latest", async () => {
    const hash = await sdk.rpc.chain.getBlockHash(await sdk.rpc.chain.getBestBlockNumber());
    expect(hash.value).toBeTruthy();
});

test("getBlockHash - latest + 1", async () => {
    const hash = await sdk.rpc.chain.getBlockHash(await sdk.rpc.chain.getBestBlockNumber() + 1);
    expect(hash).toBe(null);
});
