import { SDK } from "../";
import { payment } from "./helper";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK({ server: SERVER_URL });

test("getParcel", async () => {
    const { SignedParcel } = SDK.Core.classes;
    const hash = await payment();
    const parcel = await sdk.rpc.chain.getParcel(hash);
    expect(parcel).toEqual(expect.any(SignedParcel));
});
