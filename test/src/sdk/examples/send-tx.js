const SDK = require("codechain-sdk");

const sdk = new SDK({
    server: process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080",
    networkId: process.env.CODECHAIN_NETWORK_ID || "tc"
});

const ACCOUNT_ADDRESS =
    process.env.ACCOUNT_ADDRESS ||
    "tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd";
const ACCOUNT_PASSPHRASE = process.env.ACCOUNT_PASSPHRASE || "satoshi";

const tx = sdk.core.createPayTransaction({
    recipient: "tccqxv9y4cw0jwphhu65tn4605wadyd2sxu5yezqghw",
    quantity: 10000
});

(async () => {
    const result = await sdk.rpc.account.sendTransaction({
        tx,
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });
    const transaction = await sdk.rpc.chain.containsTransaction(result.hash);
    console.log(transaction); // true
})().catch(console.error);
