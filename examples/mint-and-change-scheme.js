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
    const aliceAddress = await sdk.key.createAssetTransferAddress();
    const bobAddress = await sdk.key.createPlatformAddress();
    const carolAddress = "tccq9qvruafmf9vegjhkl0ruunkwp0d4lc8fgxknzh5";

    // Create asset named Gold. Total supply of Gold is 10000. The approver is set
    // to null, which means this type of asset can be transferred freely.
    const goldAssetScheme = sdk.core.createAssetScheme({
        shardId,
        metadata: {
            name: "Gold",
            description: "An asset example",
            icon_url: "https://gold.image/"
        },
        supply: 10000,
        registrar: ACCOUNT_ADDRESS
    });
    const mintTx = sdk.core.createMintAssetTransaction({
        scheme: goldAssetScheme,
        recipient: aliceAddress
    });

    await sdk.rpc.chain.sendTransaction(mintTx, {
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

    await sdk.rpc.chain.sendTransaction(
        sdk.core.createPayTransaction({
            recipient: bobAddress,
            quantity: 1
        }),
        {
            account: ACCOUNT_ADDRESS,
            passphrase: ACCOUNT_PASSPHRASE
        }
    );
    await sdk.rpc.chain.sendTransaction(
        sdk.core.createPayTransaction({
            recipient: carolAddress,
            quantity: 1
        }),
        {
            account: ACCOUNT_ADDRESS,
            passphrase: ACCOUNT_PASSPHRASE
        }
    );

    const assetSchemeChangeTx = sdk.core.createChangeAssetSchemeTransaction({
        assetType: mintTx.getMintedAsset().assetType,
        shardId,
        scheme: {
            metadata: {
                name: "Golden Coin",
                description: "An asset example",
                icon_url: "https://gold.image/"
            },
            approver: bobAddress,
            registrar: carolAddress
        }
    });

    await sdk.rpc.chain.sendTransaction(assetSchemeChangeTx, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });

    const assetSchemeChangeTxResults = await sdk.rpc.chain.getTransactionResultsByTracker(
        assetSchemeChangeTx.tracker(),
        {
            timeout: 300 * 1000
        }
    );
    if (!assetSchemeChangeTxResults[0]) {
        throw Error(
            `AssetSchemeChange failed: ${JSON.stringify(
                assetSchemeChangeTxResults[0].error
            )}`
        );
    }

    console.log(
        await sdk.rpc.chain.getAssetSchemeByTracker(mintTx.tracker(), shardId)
    );
    console.log(
        await sdk.rpc.chain.getAssetSchemeByType(
            mintTx.getMintedAsset().assetType,
            shardId
        )
    );
})().catch(err => {
    console.error(`Error:`, err);
});
