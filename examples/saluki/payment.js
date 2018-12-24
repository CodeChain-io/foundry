const SDK = require("codechain-sdk");

const sdk = new SDK({
    server: "http://52.78.210.78:8080",
    networkId: "sc"
});

const parcelSender = process.env.CODECHAIN_SALUKI_ACCOUNT;
if (!sdk.core.classes.PlatformAddress.check(parcelSender)) {
    throw Error(
        "The environment variable CODECHAIN_SALUKI_ACCOUNT must be a valid platform address for Saluki. For example: sccqz8hyh3560xwpykm9u8en5k2jcwcueq6ncvg2dvy"
    );
}

(async () => {
    const keyStore = await sdk.key.createLocalKeyStore();
    const parcel = sdk.core.createPayParcel({
        recipient: "sccqywxfyz8ykulqsq2l7z9nvgd8z3cczfun509f08u",
        amount: 5 // 0.000000005CCC
    });
    const signedParcel = await sdk.key.signParcel(parcel, {
        keyStore,
        account: parcelSender,
        fee: 10,
        nonce: await sdk.rpc.chain.getNonce(parcelSender)
    });
    const parcelHash = await sdk.rpc.chain.sendSignedParcel(signedParcel);
    console.log(
        `https://saluki.codechain.io/explorer/parcel/0x${parcelHash.value}`
    );
})().catch(console.error);
