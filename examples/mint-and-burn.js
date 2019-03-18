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
        metadata: {
            name: "ExampleAsset",
            description: "This asset will be burnt shortly"
        },
        supply: 10000,
        approver: null
    });
    const mintTx = sdk.core.createMintAssetTransaction({
        scheme: assetScheme,
        recipient: aliceAddress
    });

    const firstGold = mintTx.getMintedAsset();
    const transferTx = sdk.core
        .createTransferAssetTransaction()
        .addBurns(firstGold);
    await sdk.key.signTransactionBurn(transferTx, 0);

    await sdk.rpc.chain.sendTransaction(mintTx, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });
    await sdk.rpc.chain.sendTransaction(transferTx, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });

    const mintTxResults = await sdk.rpc.chain.getTransactionResultsByTracker(
        mintTx.tracker(),
        {
            timeout: 300 * 1000
        }
    );
    if (!mintTxResults[0]) {
        throw Error(
            `AssetMintTransaction failed: ${JSON.stringify(
                mintTxResults[0].error
            )}`
        );
    }
    const transferTxResults = await sdk.rpc.chain.getTransactionResultsByTracker(
        transferTx.tracker(),
        {
            timeout: 300 * 1000
        }
    );
    if (!transferTxResults[0]) {
        throw Error(
            `AssetTransferTransaction failed: ${JSON.stringify(
                transferTxResults[0].error
            )}`
        );
    }
})().catch(err => {
    console.error(`Error:`, err);
});
