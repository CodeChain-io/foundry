import { SDK, Parcel, U256, H256, H160, PaymentTransaction } from "../";
import { privateKeyToAddress } from "../src/utils";

const SERVER_URL = "http://localhost:8080";
const sdk = new SDK(SERVER_URL);
const secret = new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd");
const address = new H160(privateKeyToAddress(secret.value));

test("sendSignedParcel", async () => {
    const nonce = await sdk.getNonce(address);
    const t = new PaymentTransaction({
        nonce: nonce.increase(),
        address,
        value: new U256(0)
    });
    const fee = new U256(10);
    const networkId = 17;
    const p = new Parcel(nonce, fee, networkId, t).sign(secret);
    const hash = await sdk.sendSignedParcel(p);
    expect(hash).toMatchObject({
        value: expect.stringMatching(/[0-9a-f]{32}/)
    });
});
