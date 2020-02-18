import { SDK } from "../";

import {
    ACCOUNT_ADDRESS,
    ACCOUNT_ID,
    ACCOUNT_SECRET,
    CODECHAIN_NETWORK_ID,
    SERVER_URL
} from "./helper";

test("getSignerAccountId", async () => {
    const sdk = new SDK({
        server: SERVER_URL,
        networkId: CODECHAIN_NETWORK_ID
    });
    const seq = await sdk.rpc.chain.getSeq(ACCOUNT_ADDRESS);
    const tx = sdk.core
        .createPayTransaction({
            quantity: 10,
            recipient: ACCOUNT_ADDRESS
        })
        .sign({
            secret: ACCOUNT_SECRET,
            fee: 10,
            seq
        });
    const hash = await sdk.rpc.chain.sendSignedTransaction(tx);
    const txReceived = await sdk.rpc.chain.getTransaction(hash);
    if (txReceived == null) {
        throw Error("Cannot get a transaction");
    }
    expect(txReceived.getSignerAccountId().value).toEqual(ACCOUNT_ID);
});
