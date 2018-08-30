const SDK = require("codechain-sdk");

const sdk = new SDK({ server: "http://localhost:8080" });

(async () => {
    const keyStore = await sdk.key.createLocalKeyStore();
    // Create P2PKHBurn instead of P2PKH. Currently, P2PKH addresses can only
    // transfer assets. If you want to burn assets which are holded by P2PKH
    // addresses, you need to transfer them to P2PKHBurn addresses first.
    const p2pkhBurn = await sdk.key.createP2PKHBurn({ keyStore });

    const aliceAddress = await p2pkhBurn.createAddress();

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
    await transferTx.signBurn(0, { signer: p2pkhBurn });

    const parcel = sdk.core.createChangeShardStateParcel({
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
