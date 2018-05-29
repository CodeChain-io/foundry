import { SDK } from "../";
import { paymentTwice } from "./helper";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK(SERVER_URL);

test("getPendingParcels", async () => {
    let pending = await sdk.getPendingParcels();
    if (pending.length === 0) {
        await paymentTwice();
    }
    pending = await sdk.getPendingParcels();
    // FIXME: test Parcel type
    expect(pending[0].transaction).toBeTruthy();
});
