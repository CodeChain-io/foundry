const SDK = require("codechain-sdk");

const sdk = new SDK({
    server: process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080",
    networkId: process.env.CODECHAIN_NETWORK_ID || "tc"
});

const ACCOUNT_ADDRESS =
    process.env.ACCOUNT_ADDRESS ||
    "tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd";

(async () => {
    const balance = await sdk.rpc.chain.getBalance(ACCOUNT_ADDRESS);
    // the balance is a U64 instance at this moment.
    // Use toString() to print it out.
    console.log(balance.toString()); // the quantity of CCC that the account has.
})().catch(console.error);
