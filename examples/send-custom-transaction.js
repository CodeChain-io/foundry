const SDK = require("codechain-sdk");

const RLP = require("rlp");

const sdk = new SDK({
    server: process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080",
    networkId: process.env.CODECHAIN_NETWORK_ID || "tc"
});

const ACCOUNT_ADDRESS =
    process.env.ACCOUNT_ADDRESS ||
    "tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd";
const ACCOUNT_PASSPHRASE = process.env.ACCOUNT_PASSPHRASE || "satoshi";

(async () => {
    const hash = await sdk.rpc.chain.sendTransaction(
        sdk.core.createCustomTransaction({
            handlerId: 1,
            bytes: RLP.encode([0])
        }),
        {
            account: ACCOUNT_ADDRESS,
            passphrase: ACCOUNT_PASSPHRASE
        }
    );
    const result = await sdk.rpc.chain.containTransaction(hash);
    console.log("result:", result);
})().catch(err => {
    console.error(`Error:`, err);
});
