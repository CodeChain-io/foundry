const SDK = require("..");

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK({
    server: SERVER_URL
});

const parcel = sdk.core.createPaymentParcel({
    recipient: "tccqxv9y4cw0jwphhu65tn4605wadyd2sxu5yezqghw",
    amount: 10000
});

(async () => {
    const keyStore = await sdk.key.createLocalKeyStore();
    const account = await sdk.key.createPlatformAddress({
        keyStore
    });
    const seq = await sdk.rpc.chain.getSeq(account);

    const signedParcel = await sdk.key.signParcel(parcel, {
        account,
        keyStore,
        fee: 10,
        seq
    });
    console.log(signedParcel);
    // FIXME: needs fee
    // const parcelHash = await sdk.rpc.chain.sendSignedParcel(signedParcel);
})();
