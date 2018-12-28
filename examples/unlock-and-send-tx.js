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
    amount: 10000
});

(async () => {
    await sdk.rpc.account.unlock(ACCOUNT_ADDRESS, ACCOUNT_PASSPHRASE);
    const result = await sdk.rpc.account.sendTransaction({
        tx,
        account: ACCOUNT_ADDRESS
    });
    const invoice = await sdk.rpc.chain.getParcelInvoice(result.hash, {
        timeout: 300 * 1000
    });
    console.log(invoice); // { success: true }
})().catch(console.error);
