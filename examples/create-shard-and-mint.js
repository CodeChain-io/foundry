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
    const account2 = await sdk.rpc.account.create("");
    await sdk.rpc.chain.sendTransaction(
        sdk.core.createPayTransaction({
            recipient: account2,
            quantity: 10000
        }),
        {
            account: ACCOUNT_ADDRESS,
            passphrase: ACCOUNT_PASSPHRASE
        }
    );

    const createShardTx = sdk.core.createCreateShardTransaction({
        users: [ACCOUNT_ADDRESS]
    });
    const createShardTxHash = await sdk.rpc.chain.sendTransaction(
        createShardTx,
        {
            account: ACCOUNT_ADDRESS,
            passphrase: ACCOUNT_PASSPHRASE
        }
    );
    const shardId = await sdk.rpc.chain.getShardIdByHash(createShardTxHash);
    if (shardId === null) {
        throw Error("CreateShard transaction failed");
    }
    console.log("shardId", shardId);

    const mintTxByNonShardUserHash = await sdk.rpc.chain.sendTransaction(
        sdk.core.createMintAssetTransaction({
            scheme: { shardId, metadata: "" },
            recipient: await sdk.key.createAssetTransferAddress()
        }),
        {
            account: account2,
            passphrase: ""
        }
    );
    const mintTxByNonShardUserResult = await sdk.rpc.chain.containTransaction(
        mintTxByNonShardUserHash
    );
    console.log(
        "MintAsset by a non-shard user result:",
        mintTxByNonShardUserResult
    );

    const mintTxByShardUserHash = await sdk.rpc.chain.sendTransaction(
        sdk.core.createMintAssetTransaction({
            scheme: { shardId, metadata: "" },
            recipient: await sdk.key.createAssetTransferAddress()
        }),
        {
            account: ACCOUNT_ADDRESS,
            passphrase: ACCOUNT_PASSPHRASE
        }
    );
    const mintTxByShardUserResult = await sdk.rpc.chain.containTransaction(
        mintTxByShardUserHash
    );
    console.log("MintAsset by a shard user result:", mintTxByShardUserResult);
})().catch(err => {
    console.error(`Error:`, err);
});
