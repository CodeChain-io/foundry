const SDK = require("codechain-sdk");

const sdk = new SDK({
    server: process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080",
    networkId: process.env.CODECHAIN_NETWORK_ID || "tc"
});

const ACCOUNT_ADDRESS =
    process.env.ACCOUNT_ADDRESS ||
    "tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd";
const ACCOUNT_PASSPHRASE = process.env.ACCOUNT_PASSPHRASE || "satoshi";

// If you want to know how to create an address, see the example "Create an
// asset address".
const address = "tcaqyqckq0zgdxgpck6tjdg4qmp52p2vx3qaexqnegylk";

const tx = sdk.core.createMintAssetTransaction({
    scheme: {
        shardId: 0,
        metadata: {
            name: "Silver Coin",
            description: "...",
            icon_url: "..."
        },
        supply: 100000000
    },
    recipient: address
});

(async () => {
    const hash = await sdk.rpc.chain.sendTransaction(tx, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });
    // Get the result of the tx.
    const result = await sdk.rpc.chain.containsTransaction(hash);
    console.log(result); // true
})().catch(console.error);
