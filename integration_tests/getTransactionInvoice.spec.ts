import { Invoice, SDK, H160, H256, U256, Parcel, PaymentTransaction, privateKeyToAddress } from "../";
import { payment } from "./helper";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK(SERVER_URL);

test("getTransactionInvoice", async () => {
    const secret = new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd");
    const address = new H160(privateKeyToAddress(secret.value));
    const nonce = await sdk.getNonce(address);
    const t = new PaymentTransaction({
        nonce: nonce.increase(),
        sender: address,
        receiver: address,
        value: new U256(0)
    });
    expect(await sdk.getTransactionInvoice(t.hash())).toBe(null);

    const fee = new U256(10);
    const networkId = 17;
    const p = new Parcel(nonce, fee, networkId, t).sign(secret);
    await sdk.sendSignedParcel(p);
    expect(await sdk.getTransactionInvoice(t.hash())).toEqual(new Invoice(true));
});
