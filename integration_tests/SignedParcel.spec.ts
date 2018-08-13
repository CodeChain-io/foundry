import { SDK } from "../";

test("getSignerAccountId", async () => {
    const sdk = new SDK({ server: process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080" });
    const secret = "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd";
    const accountId = sdk.util.getAccountIdFromPrivate(secret);
    const accountAddress = sdk.key.classes.PlatformAddress.fromAccountId(accountId);
    const nonce = await sdk.rpc.chain.getNonce(accountAddress);
    const parcelToSend = sdk.core.createPaymentParcel({
        amount: 10,
        recipient: accountAddress,
    }).sign({
        secret,
        fee: 10,
        nonce
    });
    const parcelHash = await sdk.rpc.chain.sendSignedParcel(parcelToSend);
    const parcelReceived = await sdk.rpc.chain.getParcel(parcelHash);
    expect(parcelReceived.getSignerAccountId().value).toEqual(accountId);
});
