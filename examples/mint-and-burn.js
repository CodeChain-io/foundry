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
        throw Error(
            `AssetMintTransaction failed: ${JSON.stringify(
                mintTxInvoice.error
            )}`
        );
    }
    const transferTxInvoice = await sdk.rpc.chain.getTransactionInvoice(
        transferTx.hash(),
        {
            timeout: 300 * 1000
        }
    );
    if (transferTxInvoice.success === false) {
        throw Error(
            `AssetTransferTransaction failed: ${JSON.stringify(
                transferTxInvoice.error
            )}`
        );
    }
})().catch(err => {
    console.error(`Error:`, err);
});
