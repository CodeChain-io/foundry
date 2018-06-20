import { SDK, Parcel, U256, H256, H160, privateKeyToAddress } from "../";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK(SERVER_URL);
const secret = new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd");
const address = new H160(privateKeyToAddress(secret.value));

test("sendSignedParcel", async () => {
    const nonce = await sdk.getNonce(address);
    const fee = new U256(10);
    const networkId = 17;
    const p = Parcel.payment(nonce, fee, networkId, address, new U256(0));
    const hash = await sdk.sendSignedParcel(p.sign(secret));
    expect(hash).toMatchObject({
        value: expect.stringMatching(/[0-9a-f]{32}/)
    });
});

test("sendSignedParcel - empty", async () => {
    const nonce = await sdk.getNonce(address);
    const fee = new U256(10);
    const networkId = 17;
    const p = Parcel.transactions(nonce, fee, networkId);
    const hash = await sdk.sendSignedParcel(p.sign(secret));
    expect(hash).toMatchObject({
        value: expect.stringMatching(/[0-9a-f]{32}/)
    });
});
