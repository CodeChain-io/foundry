var SDK = require("codechain-sdk");

var sdk = new SDK({
    server: process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080",
    networkId: process.env.CODECHAIN_NETWORK_ID || "tc"
});

var ACCOUNT_ADDRESS =
    process.env.ACCOUNT_ADDRESS ||
    "tccqzn9jjm3j6qg69smd7cn0eup4w7z2yu9my9a2k78";
var ACCOUNT_PASSPHRASE = process.env.ACCOUNT_PASSPHRASE || "satoshi";

var parcel = sdk.core.createPaymentParcel({
    recipient: "tccqruq09sfgax77nj4gukjcuq69uzeyv0jcs7vzngg",
    amount: 10000
});

sdk.rpc.chain
    .sendParcel(parcel, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
        // fee and nonce are optional
    })
    .then(function(parcelHash) {
        return sdk.rpc.chain.getParcelInvoice(parcelHash, {
            timeout: 300 * 1000
        });
    })
    .then(function(parcelInvoice) {
        console.log(parcelInvoice); // { success: true }
    })
    .catch(console.error);
