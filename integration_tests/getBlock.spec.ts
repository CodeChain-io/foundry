import { SDK } from "../";
import { payment } from "./helper";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK({ server: SERVER_URL });

test("getBlock - by hash", async () => {
    const { H160, H256, U256 } = SDK.Core.classes;
    await payment();
    const latest = await sdk.rpc.chain.getBestBlockNumber();
    const hash = await sdk.rpc.chain.getBlockHash(latest);
    const block = await sdk.rpc.chain.getBlock(hash);
    expect(block).toMatchObject({
        // FIXME: test timestamp, number, extraData, seal, parcels
        parentHash: expect.any(H256),
        author: expect.any(H160),
        parcelsRoot: expect.any(H256),
        stateRoot: expect.any(H256),
        invoicesRoot: expect.any(H256),
        score: expect.any(U256),
        hash: expect.any(H256),
    });
});

test("getBlock - by number", async () => {
    const { H160, H256, U256 } = SDK.Core.classes;
    await payment();
    const latest = await sdk.rpc.chain.getBestBlockNumber();
    const block = await sdk.rpc.chain.getBlock(latest);
    expect(block).toMatchObject({
        // FIXME: test timestamp, number, extraData, seal, parcels
        parentHash: expect.any(H256),
        author: expect.any(H160),
        parcelsRoot: expect.any(H256),
        stateRoot: expect.any(H256),
        invoicesRoot: expect.any(H256),
        score: expect.any(U256),
        hash: expect.any(H256),
    });
});

