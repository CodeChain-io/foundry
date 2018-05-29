import { SDK, PaymentTransaction } from "../";
import { paymentTwice } from "./helper";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK(SERVER_URL);

test.skip("getPendingParcels", async () => {
    let pending = await sdk.getPendingParcels();
    if (pending.length === 0) {
        await paymentTwice();
    }
    pending = await sdk.getPendingParcels();
    expect(pending[0].unsigned.transactions[0]).toBe(expect.any(PaymentTransaction));
});
