import { SDK } from "../";

import {
    ACCOUNT_ADDRESS,
    ACCOUNT_SECRET,
    CODECHAIN_NETWORK_ID,
    SERVER_URL
} from "./helper";

const U64 = SDK.Core.classes.U64;

const sdk = new SDK({ server: SERVER_URL, networkId: CODECHAIN_NETWORK_ID });

const masterSecret = sdk.util.generatePrivateKey();
const masterAccountId = SDK.util.getAccountIdFromPrivate(masterSecret);
const masterAddress = sdk.core.classes.PlatformAddress.fromAccountId(
    masterAccountId,
    { networkId: "tc" }
);

const regularSecret = SDK.util.generatePrivateKey();
const regularPublic = SDK.util.getPublicFromPrivate(regularSecret);

test("setRegularKey", async () => {
    await sdk.rpc.chain.sendSignedTransaction(
        sdk.core
            .createPayTransaction({
                recipient: masterAddress,
                quantity: 1000
            })
            .sign({
                secret: ACCOUNT_SECRET,
                seq: await sdk.rpc.chain.getSeq(ACCOUNT_ADDRESS),
                fee: 10
            })
    );

    const seq = await sdk.rpc.chain.getSeq(masterAddress);
    const p = sdk.core.createSetRegularKeyTransaction({
        key: regularPublic
    });
    const hash = await sdk.rpc.chain.sendSignedTransaction(
        p.sign({
            secret: masterSecret,
            seq,
            fee: 10
        })
    );
    expect(hash).toMatchObject({
        value: expect.stringMatching(/[0-9a-f]{32}/)
    });

    await sdk.rpc.chain.getInvoice(hash, { timeout: 60 * 60 * 1000 });

    const beforeBalance = await sdk.rpc.chain.getBalance(masterAddress);

    const seq2 = await sdk.rpc.chain.getSeq(masterAddress);
    const p2 = sdk.core.createPayTransaction({
        recipient: masterAddress,
        quantity: 10
    });
    const hash2 = await sdk.rpc.chain.sendSignedTransaction(
        p2.sign({
            secret: regularSecret,
            seq: seq2,
            fee: 10
        })
    );
    await sdk.rpc.chain.getInvoice(hash2, { timeout: 60 * 60 * 1000 });
    const afterBalance = await sdk.rpc.chain.getBalance(masterAddress);
    expect(afterBalance.toString()).toEqual(
        U64.minus(beforeBalance, 10).toString()
    );
});
