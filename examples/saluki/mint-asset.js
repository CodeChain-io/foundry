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
        metadata: JSON.stringify({
            name: "Saluki Coin",
            icon_url:
                "https://upload.wikimedia.org/wikipedia/commons/3/31/Red_Smooth_Saluki.jpg"
        }),
        amount: 500
    },
    recipient:
        "scaqqqhzqech2kg7mmsf942xvn40cesxrgck0xxt9gpx6336xkeglwtdsqlg8lhr"
});

const parcel = sdk.core.createAssetTransactionGroupParcel({
    transactions: [mintTx]
});

sdk.key
    .createLocalKeyStore()
    .then(async keyStore => {
        const signedParcel = await sdk.key.signParcel(parcel, {
            keyStore,
            account: parcelSender,
            fee: 10,
            nonce: await sdk.rpc.chain.getNonce(parcelSender)
        });
        const parcelHash = await sdk.rpc.chain.sendSignedParcel(signedParcel);
        console.log(
            "https://saluki.codechain.io/explorer/parcel/0x" + parcelHash.value
        );
    })
    .catch(e => {
        console.error(e);
    });
