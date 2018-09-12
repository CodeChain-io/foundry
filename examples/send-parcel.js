var SDK = require("codechain-sdk");

var SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
var sdk = new SDK({
    server: SERVER_URL
});

var parcel = sdk.core.createPaymentParcel({
    recipient: "tccqruq09sfgax77nj4gukjcuq69uzeyv0jcs7vzngg",
    amount: 10000
});

sdk.rpc.chain
    .sendParcel(parcel, {
        account: "tccqzn9jjm3j6qg69smd7cn0eup4w7z2yu9my9a2k78",
        passphrase: "satoshi"
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
