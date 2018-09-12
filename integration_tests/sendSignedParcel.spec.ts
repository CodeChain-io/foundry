import { SDK } from "../";

import { ACCOUNT_ADDRESS, ACCOUNT_SECRET, SERVER_URL } from "./helper";

const sdk = new SDK({ server: SERVER_URL });

const secret = ACCOUNT_SECRET;
const address = ACCOUNT_ADDRESS;

test("sendSignedParcel", async () => {
    const nonce = await sdk.rpc.chain.getNonce(address);
    const p = sdk.core.createPaymentParcel({
        recipient: address,
        amount: 0
    });
    const hash = await sdk.rpc.chain.sendSignedParcel(
        p.sign({
            secret,
            nonce,
            fee: 10
        })
    );
    expect(hash).toMatchObject({
        value: expect.stringMatching(/[0-9a-f]{32}/)
    });
});

test("sendSignedParcel - empty", async () => {
    const nonce = await sdk.rpc.chain.getNonce(address);
    const p = sdk.core.createAssetTransactionGroupParcel({
        transactions: []
    });
    const hash = await sdk.rpc.chain.sendSignedParcel(
        p.sign({
            secret,
            nonce,
            fee: 10
        })
    );
    expect(hash).toMatchObject({
        value: expect.stringMatching(/[0-9a-f]{32}/)
    });
});
