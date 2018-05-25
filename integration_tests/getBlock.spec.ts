import { SDK, Parcel, U256, H160, H256 } from "../";
import { payment } from "./helper";

const SERVER_URL = "http://localhost:8080";
const sdk = new SDK(SERVER_URL);

test("getBlock", async () => {
    await payment();
    const latest = await sdk.getBlockNumber();
    const hash = await sdk.getBlockHash(latest);
    const block = await sdk.getBlock(hash);
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

