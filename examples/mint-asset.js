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
// asset transfer address".
const address = "tcaqyqckq0zgdxgpck6tjdg4qmp52p2vx3qaexqnegylk";

const tx = sdk.core.createMintAssetTransaction({
    scheme: {
        shardId: 0,
        metadata: JSON.stringify({
            name: "Silver Coin",
            description: "...",
            icon_url: "..."
        }),
        amount: 100000000
    },
    recipient: address
});

(async () => {
    const hash = await sdk.rpc.chain.sendTransaction(tx, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });
    // Get the invoice of the tx.
    const invoice = await sdk.rpc.chain.getInvoice(hash, {
        // Wait up to 120 seconds to get the invoice.
        timeout: 120 * 1000
    });
    // The invoice of asset-transaction-group tx is an array of the object that has
    // type { success: boolean }. Each object represents the result of each
    // transaction.
    console.log(invoice); // [{ success: true }]
})().catch(console.error);
