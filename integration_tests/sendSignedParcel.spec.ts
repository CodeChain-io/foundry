import { SDK } from "../";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK({ server: SERVER_URL });
const secret = "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd";
const address = SDK.util.getAccountIdFromPrivate(secret);

test("sendSignedParcel", async () => {
    const nonce = await sdk.rpc.chain.getNonce(address);
    const p = sdk.core.createPaymentParcel({
        recipient: sdk.key.classes.PlatformAddress.fromAccountId(address),
        amount: 0,
    });
    const hash = await sdk.rpc.chain.sendSignedParcel(p.sign({
        secret,
        nonce,
        fee: 10
    }));
    expect(hash).toMatchObject({
        value: expect.stringMatching(/[0-9a-f]{32}/)
    });
});

test("sendSignedParcel - empty", async () => {
    const nonce = await sdk.rpc.chain.getNonce(address);
    const p = sdk.core.createChangeShardStateParcel({
        transactions: [],
    });
    const hash = await sdk.rpc.chain.sendSignedParcel(p.sign({
        secret,
        nonce,
        fee: 10
    }));
    expect(hash).toMatchObject({
        value: expect.stringMatching(/[0-9a-f]{32}/)
    });
});
