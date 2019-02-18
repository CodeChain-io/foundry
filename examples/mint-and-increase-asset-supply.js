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
    const bobAddress = await sdk.key.createAssetTransferAddress({
        type: "P2PKH"
    });
    // Create asset named Gold. Total supply of Gold is 0xfff. The approver is set
    // to null, which means this type of asset can be transferred freely.
    const goldAssetScheme = sdk.core.createAssetScheme({
        shardId,
        metadata: JSON.stringify({
            name: "Gold",
            description: "An asset example",
            icon_url: "https://gold.image/"
        }),
        supply: 0xfff,
        administrator: ACCOUNT_ADDRESS
    });
    const mintTx = sdk.core.createMintAssetTransaction({
        scheme: goldAssetScheme,
        recipient: aliceAddress
    });

    await sdk.rpc.chain.sendTransaction(mintTx, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });

    const mintTxInvoices = await sdk.rpc.chain.getInvoicesByTracker(
        mintTx.tracker(),
        {
            timeout: 300 * 1000
        }
    );
    if (!mintTxInvoices[0]) {
        throw Error(
            `AssetMintTransaction failed: ${JSON.stringify(
                mintTxInvoices[0].error
            )}`
        );
    }
    const increaseAssetSupplyTx = sdk.core.createIncreaseAssetSupplyTransaction(
        {
            shardId,
            assetType: mintTx.getMintedAsset().assetType,
            recipient: bobAddress,
            supply: 0xff0000,
            approvals: undefined
        }
    );

    await sdk.rpc.chain.sendTransaction(increaseAssetSupplyTx, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });

    const increaseAssetSupplyTxInvoices = await sdk.rpc.chain.getInvoicesByTracker(
        increaseAssetSupplyTx.tracker(),
        {
            timeout: 300 * 1000
        }
    );

    if (!increaseAssetSupplyTxInvoices[0]) {
        throw Error(
            `increaseAssetSupply failed: ${JSON.stringify(
                increaseAssetSupplyTxInvoices[0].error
            )}`
        );
    }

    console.log(
        await sdk.rpc.chain.getAssetSchemeByTracker(mintTx.tracker(), shardId)
    );

    console.log(await sdk.rpc.chain.getAsset(mintTx.tracker(), 0, shardId));

    console.log(
        await sdk.rpc.chain.getAsset(
            increaseAssetSupplyTx.tracker(),
            0,
            shardId
        )
    );
})().catch(err => {
    console.error(`Error:`, err);
});
