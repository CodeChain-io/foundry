var SDK = require("codechain-sdk");

var SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
var sdk = new SDK({
    server: SERVER_URL
});

var ACCOUNT_SECRET =
    process.env.ACCOUNT_SECRET ||
    "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd";
var ACCOUNT_ADDRESS =
    process.env.ACCOUNT_ADDRESS ||
    "tccqzn9jjm3j6qg69smd7cn0eup4w7z2yu9my9a2k78";

var parcel = sdk.core.createPaymentParcel({
    recipient: "tccqruq09sfgax77nj4gukjcuq69uzeyv0jcs7vzngg",
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
