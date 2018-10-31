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
        type: "P2PKHBurn"
    });

    // Create an asset.
    const assetScheme = sdk.core.createAssetScheme({
        shardId: 0,
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

    const mintParcel = sdk.core.createAssetTransactionParcel({
        transaction: mintTx
    });
    await sdk.rpc.chain.sendParcel(mintParcel, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });
    const transferParcel = sdk.core.createAssetTransactionParcel({
        transaction: transferTx
    });
    await sdk.rpc.chain.sendParcel(transferParcel, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });

    const mintTxInvoices = await sdk.rpc.chain.getTransactionInvoices(
        mintTx.hash(),
        {
            timeout: 300 * 1000
        }
    );
    if (!mintTxInvoices[0].success) {
        throw Error(
            `AssetMintTransaction failed: ${JSON.stringify(
                mintTxInvoices[0].error
            )}`
        );
    }
    const transferTxInvoices = await sdk.rpc.chain.getTransactionInvoices(
        transferTx.hash(),
        {
            timeout: 300 * 1000
        }
    );
    if (!transferTxInvoices[0].success) {
        throw Error(
            `AssetTransferTransaction failed: ${JSON.stringify(
                transferTxInvoices[0].error
            )}`
        );
    }
})().catch(err => {
    console.error(`Error:`, err);
});
