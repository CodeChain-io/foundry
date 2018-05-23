import { SDK } from "../";
import { sendNoopTwice } from "./helper";

const SERVER_URL = "http://localhost:8080";
const sdk = new SDK(SERVER_URL);

test("getPendingParcels", async () => {
    await sendNoopTwice();
    const pending = await sdk.getPendingParcels();
    expect(pending[0].transaction).toEqual("noop");
});