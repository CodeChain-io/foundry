var SDK = require("codechain-sdk");

var sdk = new SDK({ server: "http://localhost:8080" });

var parcel = sdk.core.createPaymentParcel({
    recipient: "cccqrjgmsqfcj9xx2frftwq5w3esgwahtkf5q0ys4jg",
    amount: 10000
});

sdk.rpc.chain.sendParcel(parcel, {
    account: "cccqzn9jjm3j6qg69smd7cn0eup4w7z2yu9myd6c4d7",
    passphrase: "satoshi"
}).then(function (parcelHash) {
    return sdk.rpc.chain.getParcelInvoice(parcelHash, { timeout: 5 * 60 * 1000 });
}).then(function (parcelInvoice) {
    console.log(parcelInvoice) // { success: true }
});
