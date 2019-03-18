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

const mintTx = sdk.core.createAssetMintTransaction({
    scheme: {
        shardId: 0,
        worldId: 0,
        metadata: {
            name: "Saluki Coin",
            icon_url:
                "https://upload.wikimedia.org/wikipedia/commons/3/31/Red_Smooth_Saluki.jpg"
        },
        supply: 500
    },
    recipient: "scaqyqa6lghs8a5mgt8e08tncmgeh55arh0rcqqm0ylk2"
});

const parcel = sdk.core.createAssetTransactionGroupParcel({
    transactions: [mintTx]
});

(async () => {
    const keyStore = await sdk.key.createLocalKeyStore();
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
