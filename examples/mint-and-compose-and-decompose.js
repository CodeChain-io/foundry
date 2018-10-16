const SDK = require("codechain-sdk");

const sdk = new SDK({
    server: process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080",
    networkId: process.env.CODECHAIN_NETWORK_ID || "tc"
});

const ACCOUNT_ADDRESS =
    process.env.ACCOUNT_ADDRESS ||
    "tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd";
const ACCOUNT_PASSPHRASE = "satoshi";

(async () => {
    const aliceAddress = await sdk.key.createAssetTransferAddress({
        type: "P2PKH"
    });

    const assetScheme = sdk.core.createAssetScheme({
        shardId: 0,
        worldId: 0,
        metadata: JSON.stringify({
            name: "An example asset"
        }),
        amount: 10,
        registrar: null
    });
    const mintTx = sdk.core.createAssetMintTransaction({
        scheme: assetScheme,
        recipient: aliceAddress
    });

    const firstAsset = mintTx.getMintedAsset();
    const composeTx = sdk.core.createAssetComposeTransaction({
        scheme: {
            shardId: 0,
            worldId: 0,
            metadata: JSON.stringify({ name: "An unique asset" }),
            amount: 1
        },
        inputs: [firstAsset.createTransferInput()],
        recipient: aliceAddress
    });
    await sdk.key.signTransactionInput(composeTx, 0);

    const composedAsset = composeTx.getComposedAsset();
    const decomposeTx = sdk.core.createAssetDecomposeTransaction({
        input: composedAsset.createTransferInput()
    });
    decomposeTx.addOutputs({
        assetType: firstAsset.assetType,
        amount: 10,
        recipient: aliceAddress
    });
    await sdk.key.signTransactionInput(decomposeTx, 0);

    const parcel = sdk.core.createAssetTransactionGroupParcel({
        transactions: [mintTx, composeTx, decomposeTx]
    });
    await sdk.rpc.chain.sendParcel(parcel, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
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
    const composeTxInvoice = await sdk.rpc.chain.getTransactionInvoice(
        composeTx.hash(),
        {
            timeout: 300 * 1000
        }
    );
    if (composeTxInvoice.success === false) {
        throw Error("AssetComposeTransaction failed");
    }
    const decomposeTxInvoice = await sdk.rpc.chain.getTransactionInvoice(
        decomposeTx.hash(),
        {
            timeout: 300 * 1000
        }
    );
    if (decomposeTxInvoice.success === false) {
        throw Error("AssetDecomposeTransaction failed");
    }
})().catch(err => {
    console.error(`Error:`, err);
});
