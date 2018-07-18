var SDK = require("codechain-sdk");

var sdk = new SDK({ server: "http://localhost:8080" });

var signerAddress = "0xa6594b7196808d161b6fb137e781abbc251385d9";
var signerSecret = "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd"

var recipientAddress = "0x744142069fe2d03d48e61734cbe564fcc94e6e31";

sdk.rpc.chain.getNonce(signerAddress).then(function (nonce) {
    var parcel = sdk.core.createPaymentParcel({
        recipient: recipientAddress,
        amount: 10000,
    });
    var signedParcel = parcel.sign({
        secret: signerSecret,
        nonce,
        fee: 10
    });
    return sdk.rpc.chain.sendSignedParcel(signedParcel);
}).then(function (parcelHash) {
    console.log(`Parcel sent:`, parcelHash);
    return sdk.rpc.chain.getParcel(parcelHash);
}).then(function (parcel) {
    console.log(`Parcel`, parcel);
}).catch((err) => {
    console.error(`Error:`, err);
});
