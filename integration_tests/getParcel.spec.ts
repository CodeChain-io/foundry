import { SDK, Parcel, U256 } from "../";
import { payment } from "./helper";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK(SERVER_URL);

test("getParcel", async () => {
    const hash = await payment();
    const parcel = await sdk.getParcel(hash);
    expect(parcel).toMatchObject({
        fee: expect.any(U256),
        nonce: expect.any(U256),
        networkId: expect.any(U256),
        transaction: expect.objectContaining({
            type: expect.anything(),
            data: expect.anything(),
        }),
    });
});

