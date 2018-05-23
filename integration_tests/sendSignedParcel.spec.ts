import { SDK, NoopTransaction, Parcel, U256, H256, H160 } from "../";
import { privateKeyToAddress } from "../src/utils";

const SERVER_URL = "http://localhost:8080";
const sdk = new SDK(SERVER_URL);
const secret = new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd");
const address = new H160(privateKeyToAddress(secret.value));

test("sendSignedParcel", async () => {
    const t = new NoopTransaction();
    const nonce = await sdk.getNonce(address);
    const fee = new U256(10);
    const networkId = 17;
    const p = new Parcel(nonce, fee, t, networkId);
    sdk.sendSignedParcel(p.sign(secret)).then(res => {
        expect(res).toMatchObject({
            value: expect.stringMatching(/[0-9a-f]{32}/)
        });
    });
});
