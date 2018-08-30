const SDK = require("codechain-sdk");

const sdk = new SDK({ server: "http://localhost:8080" });

(async () => {
    const keyStore = await sdk.key.createLocalKeyStore();
    const p2pkh = await sdk.key.createP2PKH({ keyStore });

    const aliceAddress = await p2pkh.createAddress();
    const bobAddress =
        "tcaqqq9pgkq69z488qlkvhkpcxcgfd3cqlkzgxyq9cewxuda8qqz7jtlvctt5eze";

    // Create asset named Gold. Total amount of Gold is 10000. The registrar is set
    // to null, which means this type of asset can be transferred freely.
    const goldAssetScheme = sdk.core.createAssetScheme({
        shardId: 0,
        worldId: 0,
        metadata: JSON.stringify({
            name: "Gold",
            description: "An asset example",
            icon_url: "https://gold.image/"
        }),
        amount: 10000,
        registrar: null
    });
    const mintTx = sdk.core.createAssetMintTransaction({
        scheme: goldAssetScheme,
        recipient: aliceAddress
    });

    const firstGold = mintTx.getMintedAsset();
    const transferTx = sdk.core
        .createAssetTransferTransaction()
        .addInputs(firstGold)
        .addOutputs(
            {
                recipient: bobAddress,
                amount: 3000,
                assetType: firstGold.assetType
            },
            {
                recipient: aliceAddress,
                amount: 7000,
                assetType: firstGold.assetType
            }
        );
    await transferTx.signInput(0, { signer: p2pkh });

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

    // Unspent Bob's 3000 golds
    console.log(await sdk.rpc.chain.getAsset(transferTx.hash(), 0));
    // Unspent Alice's 7000 golds
    console.log(await sdk.rpc.chain.getAsset(transferTx.hash(), 1));
})().catch(err => {
    console.error(`Error:`, err);
});
