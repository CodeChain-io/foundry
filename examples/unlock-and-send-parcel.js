var SDK = require("codechain-sdk");

var sdk = new SDK({
    server: process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080",
    networkId: process.env.CODECHAIN_NETWORK_ID || "tc"
});

var ACCOUNT_ADDRESS =
    process.env.ACCOUNT_ADDRESS ||
    "tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd";
var ACCOUNT_PASSPHRASE = process.env.ACCOUNT_PASSPHRASE || "satoshi";

var parcel = sdk.core.createPaymentParcel({
    recipient: "tccqxv9y4cw0jwphhu65tn4605wadyd2sxu5yezqghw",
    amount: 10000
});

sdk.rpc.account
    .unlock(ACCOUNT_ADDRESS, ACCOUNT_PASSPHRASE)
    .then(() => {
        return sdk.rpc.account
            .sendParcel({
                parcel,
                account: ACCOUNT_ADDRESS
            })
            .then(function(result) {
                return sdk.rpc.chain.getParcelInvoice(result.hash, {
                    timeout: 300 * 1000
                });
            })
            .then(function(parcelInvoice) {
                console.log(parcelInvoice); // { success: true }
            });
    })
    .catch(console.error);
