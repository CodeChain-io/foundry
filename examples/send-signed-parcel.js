var SDK = require("codechain-sdk");

var sdk = new SDK({
    server: process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080",
    networkId: process.env.CODECHAIN_NETWORK_ID || "tc"
});

var ACCOUNT_SECRET =
    process.env.ACCOUNT_SECRET ||
    "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd";
var ACCOUNT_ADDRESS =
    process.env.ACCOUNT_ADDRESS ||
    "tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd";

var parcel = sdk.core.createPaymentParcel({
    recipient: "tccqxv9y4cw0jwphhu65tn4605wadyd2sxu5yezqghw",
    amount: 10000
});

sdk.rpc.chain
    .getNonce(ACCOUNT_ADDRESS)
    .then(function(nonce) {
        return sdk.rpc.chain.sendSignedParcel(
            parcel.sign({
                secret: ACCOUNT_SECRET,
                fee: 10,
                nonce: nonce
            })
        );
    })
    .then(function(parcelHash) {
        return sdk.rpc.chain.getParcelInvoice(parcelHash, {
            timeout: 300 * 1000
        });
    })
    .then(function(parcelInvoice) {
        console.log(parcelInvoice); // { success: true }
    });
