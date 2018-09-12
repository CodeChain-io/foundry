import { SDK } from "../";

import { ACCOUNT_ADDRESS, ACCOUNT_SECRET, SERVER_URL } from "./helper";

const U256 = SDK.Core.classes.U256;

const sdk = new SDK({ server: SERVER_URL });

const masterSecret = ACCOUNT_SECRET;
const masterAddress = ACCOUNT_ADDRESS;

const regularSecret = SDK.util.generatePrivateKey();
const regularPublic = SDK.util.getPublicFromPrivate(regularSecret);

test("setRegularKey", async () => {
    const nonce = await sdk.rpc.chain.getNonce(masterAddress);
    const p = sdk.core.createSetRegularKeyParcel({
        key: regularPublic
    });
    const hash = await sdk.rpc.chain.sendSignedParcel(
        p.sign({
            secret: masterSecret,
            nonce,
            fee: 10
        })
    );
    expect(hash).toMatchObject({
        value: expect.stringMatching(/[0-9a-f]{32}/)
    });

    await sdk.rpc.chain.getParcelInvoice(hash, { timeout: 60 * 60 * 1000 });

    const beforeBalance = await sdk.rpc.chain.getBalance(masterAddress);

    const nonce2 = await sdk.rpc.chain.getNonce(masterAddress);
    const p2 = sdk.core.createPaymentParcel({
        recipient: masterAddress,
        amount: 10
    });
    const hash2 = await sdk.rpc.chain.sendSignedParcel(
        p2.sign({
            secret: regularSecret,
            nonce: nonce2,
            fee: 10
        })
    );
    await sdk.rpc.chain.getParcelInvoice(hash2, { timeout: 60 * 60 * 1000 });
    const afterBalance = await sdk.rpc.chain.getBalance(masterAddress);
    expect(afterBalance.toString()).toEqual(
        new U256(beforeBalance.value.minus(10)).toString()
    );
});
