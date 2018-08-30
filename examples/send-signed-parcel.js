var SDK = require("codechain-sdk");

var sdk = new SDK({ server: "http://localhost:8080" });

var parcel = sdk.core.createPaymentParcel({
    recipient: "tccqruq09sfgax77nj4gukjcuq69uzeyv0jcs7vzngg",
    amount: 10000
});

var account = "tccqzn9jjm3j6qg69smd7cn0eup4w7z2yu9my9a2k78";
var accountSecret =
    "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd";

sdk.rpc.chain
    .getNonce(account)
    .then(function(nonce) {
        return sdk.rpc.chain.sendSignedParcel(
            parcel.sign({
                secret: accountSecret,
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
