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
        metadata: JSON.stringify({
            name: "An example asset"
        }),
        amount: 10,
        approver: null
    });
    const mintTx = sdk.core.createAssetMintTransaction({
        scheme: assetScheme,
        recipient: aliceAddress
    });

    const firstAsset = mintTx.getMintedAsset();
    const composeTx = sdk.core.createAssetComposeTransaction({
        scheme: {
            shardId: 0,
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

    const mintParcel = sdk.core.createAssetTransactionParcel({
        transaction: mintTx
    });
    await sdk.rpc.chain.sendParcel(mintParcel, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });
    const composeParcel = sdk.core.createAssetTransactionParcel({
        transaction: composeTx
    });
    await sdk.rpc.chain.sendParcel(composeParcel, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });
    const decomposeParcel = sdk.core.createAssetTransactionParcel({
        transaction: decomposeTx
    });
    await sdk.rpc.chain.sendParcel(decomposeParcel, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });

    const mintTxInvoices = await sdk.rpc.chain.getTransactionInvoices(
        mintTx.id(),
        {
            timeout: 300 * 1000
        }
    );
    if (!mintTxInvoices[0].success) {
        throw Error("AssetMintTransaction failed");
    }
    const composeTxInvoices = await sdk.rpc.chain.getTransactionInvoices(
        composeTx.id(),
        {
            timeout: 300 * 1000
        }
    );
    if (!composeTxInvoices[0].success) {
        throw Error("AssetComposeTransaction failed");
    }
    const decomposeTxInvoices = await sdk.rpc.chain.getTransactionInvoices(
        decomposeTx.id(),
        {
            timeout: 300 * 1000
        }
    );
    if (!decomposeTxInvoices[0].success) {
        throw Error("AssetDecomposeTransaction failed");
    }
})().catch(err => {
    console.error(`Error:`, err);
});
