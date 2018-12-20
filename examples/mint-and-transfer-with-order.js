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
    const aliceAddress = await sdk.key.createAssetTransferAddress();
    const bobAddress = await sdk.key.createAssetTransferAddress();

    const goldAssetScheme = sdk.core.createAssetScheme({
        shardId: 0,
        metadata: JSON.stringify({
            name: "Gold",
            description: "An asset example",
            icon_url: "https://gold.image/"
        }),
        amount: 10000,
        registrar: null
    });
    const goldMintTx = sdk.core.createAssetMintTransaction({
        scheme: goldAssetScheme,
        recipient: aliceAddress
    });

    const silverAssetScheme = sdk.core.createAssetScheme({
        shardId: 0,
        metadata: JSON.stringify({
            name: "Silver",
            description: "An asset example",
            icon_url: "https://silver.image/"
        }),
        amount: 100000,
        registrar: null
    });
    const silverMintTx = sdk.core.createAssetMintTransaction({
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
        assetAmountFrom: 100,
        assetAmountTo: 1000,
        expiration,
        originOutputs: [goldInput.prevOut],
        recipientFrom: aliceAddress
    });
    await sdk.key.signTransactionInputWithOrder(goldInput, order);

    /// Bob receive the order and signed input
    const transferTx = sdk.core
        .createAssetTransferTransaction()
        .addInputs(goldInput, silverInput)
        .addOutputs(
            {
                recipient: aliceAddress,
                amount: 10000 - 100,
                assetType: gold.assetType
            },
            {
                recipient: aliceAddress,
                amount: 1000,
                assetType: silver.assetType
            },
            {
                recipient: bobAddress,
                amount: 100,
                assetType: gold.assetType
            },
            {
                recipient: bobAddress,
                amount: 99000,
                assetType: silver.assetType
            }
        )
        .addOrder({
            order,
            spentAmount: 100,
            inputIndices: [0],
            outputIndices: [0, 1]
        });
    await sdk.key.signTransactionInput(transferTx, 1);

    const goldMintParcel = sdk.core.createAssetTransactionParcel({
        transaction: goldMintTx
    });
    await sdk.rpc.chain.sendParcel(goldMintParcel, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });
    const silverMintParcel = sdk.core.createAssetTransactionParcel({
        transaction: silverMintTx
    });
    await sdk.rpc.chain.sendParcel(silverMintParcel, {
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

    const goldMintTxInvoices = await sdk.rpc.chain.getTransactionInvoices(
        goldMintTx.hash(),
        {
            timeout: 300 * 1000
        }
    );
    if (!goldMintTxInvoices[0].success) {
        throw Error(
            `AssetMintTransaction failed: ${JSON.stringify(
                goldMintTxInvoices[0].error
            )}`
        );
    }
    const silverMintTxInvoices = await sdk.rpc.chain.getTransactionInvoices(
        silverMintTx.hash(),
        {
            timeout: 300 * 1000
        }
    );
    if (!silverMintTxInvoices[0].success) {
        throw Error(
            `AssetMintTransaction failed: ${JSON.stringify(
                silverMintTxInvoices[0].error
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

    // Unspent Alice's 9900 golds with the order
    console.log(await sdk.rpc.chain.getAsset(transferTx.hash(), 0));
    // 1000 silvers from Bob to Alice by the order
    console.log(await sdk.rpc.chain.getAsset(transferTx.hash(), 1));
    // 100 golds from Alice to Bob, without any order (Bob owns)
    console.log(await sdk.rpc.chain.getAsset(transferTx.hash(), 2));
    // Unspent Bob's 99000 silvers without any order
    console.log(await sdk.rpc.chain.getAsset(transferTx.hash(), 3));
})().catch(err => {
    console.error(`Error:`, err);
});
