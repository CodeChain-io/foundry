import { SDK, SignedParcel } from "../";
import { payment } from "./helper";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK(SERVER_URL);

test("getParcel", async () => {
    const hash = await payment();
    const parcel = await sdk.getParcel(hash);
    expect(parcel).toEqual(expect.any(SignedParcel));
});
