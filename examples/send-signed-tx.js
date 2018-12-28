const SDK = require("codechain-sdk");

const sdk = new SDK({
    server: process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080",
    networkId: process.env.CODECHAIN_NETWORK_ID || "tc"
});

const ACCOUNT_SECRET =
    process.env.ACCOUNT_SECRET ||
    "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd";
const ACCOUNT_ADDRESS =
    process.env.ACCOUNT_ADDRESS ||
    "tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd";

const tx = sdk.core.createPayTransaction({
    recipient: "tccqxv9y4cw0jwphhu65tn4605wadyd2sxu5yezqghw",
    amount: 10000
});

(async () => {
    const seq = await sdk.rpc.chain.getSeq(ACCOUNT_ADDRESS);
    const hash = await sdk.rpc.chain.sendSignedTransaction(
        tx.sign({
            secret: ACCOUNT_SECRET,
            fee: 10,
            seq
        })
    );
    const invoice = await sdk.rpc.chain.getParcelInvoice(hash, {
        timeout: 300 * 1000
    });
    console.log(invoice); // { success: true }
})().catch(console.error);
