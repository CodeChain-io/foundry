const SDK = require("codechain-sdk");

const sdk = new SDK({
    server: process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080",
    networkId: process.env.CODECHAIN_NETWORK_ID || "tc"
});

const ACCOUNT_SECRET =
    process.env.ACCOUNT_SECRET ||
    "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd";
const ACCOUNT_PASSPHRASE = process.env.ACCOUNT_PASSPHRASE || "satoshi";

(async () => {
    const address = await sdk.rpc.account.importRaw(
        ACCOUNT_SECRET,
        ACCOUNT_PASSPHRASE
    );
    console.log(address); // tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd
})().catch(e => {
    if (e.message !== "Already Exists") {
        console.error(e);
    }
});
