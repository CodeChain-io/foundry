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
    const account2 = await sdk.rpc.account.create();
    const createShardTxHash = await sdk.rpc.chain.sendTransaction(
        sdk.core.createCreateShardTransaction({
            users: [account2]
        }),
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

    const setShardUsersHash = await sdk.rpc.chain.sendTransaction(
        sdk.core.createSetShardUsersTransaction({
            shardId,
            users: [ACCOUNT_ADDRESS, account2]
        }),
        {
            account: ACCOUNT_ADDRESS,
            passphrase: ACCOUNT_PASSPHRASE
        }
    );
    const setShardUsersInvoice = await sdk.rpc.chain.getInvoice(
        setShardUsersHash
    );
    if (!setShardUsersInvoice) {
        throw Error("SetShardUsers transaction failed");
    }

    const shardUsers = await sdk.rpc.chain.getShardUsers(shardId);
    console.log(shardUsers);
})().catch(err => {
    console.error(`Error:`, err);
});
