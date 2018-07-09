var SDK = require("codechain-sdk");

var sdk = new SDK({ server: "http://localhost:8080" });

var signerAddress = "0xa6594b7196808d161b6fb137e781abbc251385d9";
var recipientAddress = "0x744142069fe2d03d48e61734cbe564fcc94e6e31";

// the secret key of the signer. This will be used to sign the parcel in this
// example.
var signerSecret = "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd"

// Parcel is only valid if the nonce matches the nonce of the parcel signer.
// The nonce of the signer is increased by 1 when this parcel is confirmed.
sdk.rpc.chain.getNonce(signerAddress).then(function (nonce) {
    // Create the Parcel for the payment
    var parcel = sdk.core.createPaymentParcel({
        // Recipient of the payment
        recipient: recipientAddress,
        // Amount of the payment.
        amount: 10000,
        // Nonce of the signer
        nonce,
        // Parcel signer pays 10 CCC as fee.
        fee: 10
    });
    var signedParcel = parcel.sign(signerSecret);
    // Send the signed parcel to the CodeChain node. The node will propagate this
    // parcel and attempt to confirm it.
    return sdk.rpc.chain.sendSignedParcel(signedParcel);
}).then(function (parcelHash) {
    // sendSignedParcel returns a promise that resolves with a parcel hash if parcel has
    // been verified and queued successfully. It doesn't mean parcel was confirmed.
    console.log(`Parcel sent:`, parcelHash);
    return sdk.rpc.chain.getParcel(parcelHash);
}).then(function (parcel) {
    // getParcel returns a promise that resolves with a parcel.
    // blockNumber/blockHash/parcelIndex fields in Parcel is present only for the
    // confirmed parcel
    console.log(`Parcel`, parcel);
}).catch((err) => {
    console.error(`Error:`, err);
});
