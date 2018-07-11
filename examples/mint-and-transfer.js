const SDK = require("codechain-sdk");

const sdk = new SDK({ server: "http://localhost:8080" });

// AssetAgent creates addresses for assets and manages their locking/unlocking
// data.
const assetAgent = sdk.key.getAssetAgent();

// sendTransaction() is a function to make transaction to be processed.
async function sendTransaction(tx) {
    const parcelSignerSecret = "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd";
    const parcelSignerAddress = SDK.util.getAccountIdFromPrivate(parcelSignerSecret);
    const parcel = sdk.core.createChangeShardStateParcel({
        transactions: [tx],
        nonce: await sdk.rpc.chain.getNonce(parcelSignerAddress),
        fee: 10,
    }).sign(parcelSignerSecret)
    return await sdk.rpc.chain.sendSignedParcel(parcel);
}

(async () => {
    // Create addresses for Alice and Bob.
    const aliceAddress = await assetAgent.createAddress();
    const bobAddress = await assetAgent.createAddress();

    // Create asset named Gold. Total amount of Gold is 10000. The registrar is set
    // to null, which means this type of asset can be transferred freely.
    const goldAssetScheme = sdk.core.createAssetScheme({
        metadata: JSON.stringify({
            name: "Gold",
            description: "An asset example",
            icon_url: "https://gold.image/",
        }),
        amount: 10000,
        registrar: null,
    })

    const mintTx = goldAssetScheme.mint(aliceAddress);

    await sendTransaction(mintTx);
    const mintTxInvoice = await sdk.rpc.chain.getTransactionInvoice(mintTx.hash(), 5 * 60 * 1000);
    if (!mintTxInvoice.success) {
        throw "AssetMintTransaction failed";
    }

    const firstGold = await sdk.rpc.chain.getAsset(mintTx.hash(), 0);

    const transferTx = await firstGold.transfer(assetAgent, [{
        address: bobAddress,
        amount: 3000
    }, {
        address: aliceAddress,
        amount: 7000
    }]);

    await sendTransaction(transferTx);
    const transferTxInvoice = await sdk.rpc.chain.getTransactionInvoice(transferTx.hash(), 5 * 60 * 1000);
    if (!transferTxInvoice.success) {
        throw "AssetTransferTransaction failed";
    }

    // Unspent Bob's 3000 golds
    console.log(await sdk.rpc.chain.getAsset(transferTx.hash(), 0));
    // Unspent Alice's 7000 golds
    console.log(await sdk.rpc.chain.getAsset(transferTx.hash(), 1));
})().catch((err) => {
    console.error(`Error:`, err);
});