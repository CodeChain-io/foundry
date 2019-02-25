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
    let shardId = 0;

    const aliceAddress = await sdk.key.createAssetTransferAddress({
        type: "P2PKH"
    });

    const assetScheme = sdk.core.createAssetScheme({
        shardId,
        metadata: JSON.stringify({
            name: "An example asset"
        }),
        supply: 10,
        approver: null
    });
    const mintTx = sdk.core.createMintAssetTransaction({
        scheme: assetScheme,
        recipient: aliceAddress
    });

    const firstAsset = mintTx.getMintedAsset();
    const composeTx = sdk.core.createComposeAssetTransaction({
        scheme: {
            shardId,
            metadata: JSON.stringify({ name: "An unique asset" }),
            supply: 1
        },
        inputs: [firstAsset.createTransferInput()],
        recipient: aliceAddress
    });
    await sdk.key.signTransactionInput(composeTx, 0);

    const composedAsset = composeTx.getComposedAsset();
    const decomposeTx = sdk.core.createDecomposeAssetTransaction({
        input: composedAsset.createTransferInput()
    });
    decomposeTx.addOutputs({
        assetType: firstAsset.assetType,
        shardId,
        quantity: 10,
        recipient: aliceAddress
    });
    await sdk.key.signTransactionInput(decomposeTx, 0);

    await sdk.rpc.chain.sendTransaction(mintTx, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });
    await sdk.rpc.chain.sendTransaction(composeTx, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });
    await sdk.rpc.chain.sendTransaction(decomposeTx, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });

    const mintTxResults = await sdk.rpc.chain.getTransactionResultsByTracker(
        mintTx.tracker(),
        {
            timeout: 300 * 1000
        }
    );
    if (!mintTxResults[0].success) {
        throw Error("AssetMintTransaction failed");
    }
    const composeTxResults = await sdk.rpc.chain.getTransactionResultsByTracker(
        composeTx.tracker(),
        {
            timeout: 300 * 1000
        }
    );
    if (!composeTxResults[0].success) {
        throw Error("AssetComposeTransaction failed");
    }
    const decomposeTxResults = await sdk.rpc.chain.getTransactionResultsByTracker(
        decomposeTx.tracker(),
        {
            timeout: 300 * 1000
        }
    );
    if (!decomposeTxResults[0].success) {
        throw Error("AssetDecomposeTransaction failed");
    }
})().catch(err => {
    console.error(`Error:`, err);
});
