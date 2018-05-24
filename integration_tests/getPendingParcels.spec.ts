import { SDK } from "../";
import { sendNoopTwice } from "./helper";

const SERVER_URL = "http://localhost:8080";
const sdk = new SDK(SERVER_URL);

test("getPendingParcels", async () => {
    let pending = await sdk.getPendingParcels();
    if (pending.length > 0) {
        // FIXME: test Parcel type
        expect(pending[0].transaction).toBeTruthy();
        return;
    }
    await sendNoopTwice();
    pending = await sdk.getPendingParcels();
    // FIXME: test Parcel type
    expect(pending[0].transaction).toEqual("noop");
});
