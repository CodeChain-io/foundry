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
    const nonce = await sdk.rpc.chain.getNonce(ACCOUNT_ADDRESS);
    const parcelToSend = sdk.core
        .createPaymentParcel({
            amount: 10,
            recipient: ACCOUNT_ADDRESS
        })
        .sign({
            secret: ACCOUNT_SECRET,
            fee: 10,
            nonce
        });
    const parcelHash = await sdk.rpc.chain.sendSignedParcel(parcelToSend);
    const parcelReceived = await sdk.rpc.chain.getParcel(parcelHash);
    if (parcelReceived == null) {
        throw Error("Cannot get a parcel");
    }
    expect(parcelReceived.getSignerAccountId().value).toEqual(ACCOUNT_ID);
});
