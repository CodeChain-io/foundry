const SDK = require("codechain-sdk");

const sdk = new SDK({
    server: process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080",
    networkId: process.env.CODECHAIN_NETWORK_ID || "tc"
});

const ACCOUNT_ADDRESS =
    process.env.ACCOUNT_ADDRESS ||
    "tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd";
const ACCOUNT_PASSPHRASE = process.env.ACCOUNT_PASSPHRASE || "satoshi";

(async () => {
    const shardId = 0;
    const aliceAddress = await sdk.key.createAssetAddress();
    const bobAddress = await sdk.key.createAssetAddress();

    const goldAssetScheme = sdk.core.createAssetScheme({
        shardId,
        metadata: {
            name: "Gold",
            description: "An asset example",
            icon_url: "https://gold.image/"
        },
        supply: 10000,
        registrar: null
    });
    const goldMintTx = sdk.core.createMintAssetTransaction({
        scheme: goldAssetScheme,
        recipient: aliceAddress
    });

    const silverAssetScheme = sdk.core.createAssetScheme({
        shardId,
        metadata: {
            name: "Silver",
            description: "An asset example",
            icon_url: "https://silver.image/"
        },
        supply: 100000,
        registrar: null
    });
    const silverMintTx = sdk.core.createMintAssetTransaction({
        scheme: silverAssetScheme,
        recipient: bobAddress
    });

    const gold = goldMintTx.getMintedAsset();
    const silver = silverMintTx.getMintedAsset();

    const goldInput = gold.createTransferInput();
    const silverInput = silver.createTransferInput();

    // Order is valid for 120 seconds
    const expiration = Math.round(Date.now() / 1000) + 120;
    const order = sdk.core.createOrder({
        assetTypeFrom: gold.assetType,
        assetTypeTo: silver.assetType,
        shardIdFrom: shardId,
        shardIdTo: shardId,
        assetQuantityFrom: 100,
        assetQuantityTo: 1000,
        expiration,
        originOutputs: [goldInput.prevOut],
        recipientFrom: aliceAddress
    });
    await sdk.key.signTransactionInputWithOrder(goldInput, order);

    /// Bob receive the order and signed input
    const transferTx = sdk.core
        .createTransferAssetTransaction()
        .addInputs(goldInput, silverInput)
        .addOutputs(
            {
                recipient: aliceAddress,
                quantity: 10000 - 100,
                assetType: gold.assetType,
                shardId
            },
            {
                recipient: aliceAddress,
                quantity: 1000,
                assetType: silver.assetType,
                shardId
            },
            {
                recipient: bobAddress,
                quantity: 100,
                assetType: gold.assetType,
                shardId
            },
            {
                recipient: bobAddress,
                quantity: 99000,
                assetType: silver.assetType,
                shardId
            }
        )
        .addOrder({
            order,
            spentQuantity: 100,
            inputIndices: [0],
            outputIndices: [0, 1]
        });
    await sdk.key.signTransactionInput(transferTx, 1);

    await sdk.rpc.chain.sendTransaction(goldMintTx, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });
    await sdk.rpc.chain.sendTransaction(silverMintTx, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });
    await sdk.rpc.chain.sendTransaction(transferTx, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });

    const goldMintTxResults = await sdk.rpc.chain.getTransactionResultsByTracker(
        goldMintTx.tracker(),
        {
            timeout: 300 * 1000
        }
    );
    if (!goldMintTxResults[0]) {
        throw Error(
            `AssetMintTransaction failed: ${JSON.stringify(
                goldMintTxResults[0].error
            )}`
        );
    }
    const silverMintTxResults = await sdk.rpc.chain.getTransactionResultsByTracker(
        silverMintTx.tracker(),
        {
            timeout: 300 * 1000
        }
    );
    if (!silverMintTxResults[0]) {
        throw Error(
            `AssetMintTransaction failed: ${JSON.stringify(
                silverMintTxResults[0].error
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

    // Unspent Alice's 9900 golds with the order
    console.log(await sdk.rpc.chain.getAsset(transferTx.tracker(), 0, shardId));
    // 1000 silvers from Bob to Alice by the order
    console.log(await sdk.rpc.chain.getAsset(transferTx.tracker(), 1, shardId));
    // 100 golds from Alice to Bob, without any order (Bob owns)
    console.log(await sdk.rpc.chain.getAsset(transferTx.tracker(), 2, shardId));
    // Unspent Bob's 99000 silvers without any order
    console.log(await sdk.rpc.chain.getAsset(transferTx.tracker(), 3, shardId));
})().catch(err => {
    console.error(`Error:`, err);
});
