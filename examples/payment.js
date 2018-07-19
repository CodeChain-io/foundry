var SDK = require("codechain-sdk");

var sdk = new SDK({ server: "http://localhost:8080" });

var signerSecret = "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd"

sdk.rpc.account.importRaw(signerSecret).then(account => {
    var parcel = sdk.core.createPaymentParcel({
        recipient: "0x744142069fe2d03d48e61734cbe564fcc94e6e31",
        amount: 10000,
    });
    return sdk.rpc.chain.sendParcel(parcel, { account });
}).then(function (parcelHash) {
    console.log("Parcel Hash: ", parcelHash);
    return sdk.rpc.chain.getParcelInvoice(parcelHash, { timeout: 5 * 60 * 1000 });
}).then(function (invoice) {
    console.log("Parcel Invoice: ", invoice);
}).catch((err) => {
    console.error(`Error:`, err);
});
