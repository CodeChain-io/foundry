import { SDK } from "../";
import { payment } from "./helper";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK({ server: SERVER_URL });

test.skip("getPendingParcels", async () => {
    let pending = await sdk.getPendingParcels();
    if (pending.length === 0) {
        await payment();
        await payment({ inc_nonce: 1 });
    }
    pending = await sdk.getPendingParcels();
    expect(pending[0]).toBe(expect.anything());
});
