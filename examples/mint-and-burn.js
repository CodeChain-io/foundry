const SDK = require("codechain-sdk");

const sdk = new SDK({
    server: "http://localhost:8080"
});

(async () => {
    const aliceAddress = await sdk.key.createAssetTransferAddress({
        type: "P2PKHBurn"
    });

    // Create an asset.
    const assetScheme = sdk.core.createAssetScheme({
        shardId: 0,
        worldId: 0,
        metadata: JSON.stringify({
            name: "ExampleAsset",
            description: "This asset will be burnt shortly"
        }),
        amount: 10000,
        registrar: null
    });
    const mintTx = sdk.core.createAssetMintTransaction({
        scheme: assetScheme,
        recipient: aliceAddress
    });

    const firstGold = mintTx.getMintedAsset();
    const transferTx = sdk.core
        .createAssetTransferTransaction()
        .addBurns(firstGold);
    await sdk.key.signTransactionBurn(transferTx, 0);

    const parcel = sdk.core.createAssetTransactionGroupParcel({
        transactions: [mintTx, transferTx]
    });
    await sdk.rpc.chain.sendParcel(parcel, {
        account: "tccqzn9jjm3j6qg69smd7cn0eup4w7z2yu9my9a2k78",
        passphrase: "satoshi"
    });

    const mintTxInvoice = await sdk.rpc.chain.getTransactionInvoice(
        mintTx.hash(),
        {
            timeout: 300 * 1000
        }
    );
    if (mintTxInvoice.success === false) {
        throw Error("AssetMintTransaction failed");
    }
    const transferTxInvoice = await sdk.rpc.chain.getTransactionInvoice(
        transferTx.hash(),
        {
            timeout: 300 * 1000
        }
    );
    if (transferTxInvoice.success === false) {
        throw Error("AssetTransferTransaction failed");
    }
})().catch(err => {
    console.error(`Error:`, err);
});
