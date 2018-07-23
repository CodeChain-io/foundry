var SDK = require("codechain-sdk");

var sdk = new SDK({ server: "http://localhost:8080" });

var parcel = sdk.core.createPaymentParcel({
    recipient: "0x744142069fe2d03d48e61734cbe564fcc94e6e31",
    amount: 10000
});

sdk.rpc.chain.sendParcel(parcel, {
    account: "0xa6594b7196808d161b6fb137e781abbc251385d9",
    passphrase: "satoshi"
}).then(function (parcelHash) {
    return sdk.rpc.chain.getParcelInvoice(parcelHash, { timeout: 5 * 60 * 1000 });
}).then(function (parcelInvoice) {
    console.log(parcelInvoice) // { success: true }
});
