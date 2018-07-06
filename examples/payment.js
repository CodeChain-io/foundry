const SDK = require("codechain-sdk");
const { Parcel, U256, H256, H160 } = SDK;

// Create SDK object with CodeChain RPC server URL
const sdk = new SDK({ server: "http://localhost:8080" });

const signerAddress = "0xa6594b7196808d161b6fb137e781abbc251385d9";
const recipientAddress = "0x744142069fe2d03d48e61734cbe564fcc94e6e31";

// the secret key of the signer. This will be used to sign the parcel in this
// example.
const signerSecret = "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd"

// Parcel is only valid if the nonce matches the nonce of the parcel signer.
// The nonce of the signer is increased by 1 when this parcel is confirmed.
sdk.getNonce("0xa6594b7196808d161b6fb137e781abbc251385d9").then(nonce => {
    // Create the Parcel for the payment
    const parcel = sdk.createPaymentParcel({
        // Recipient of the payment
        recipient: recipientAddress,
        // Amount of the payment.
        value: 10000,
        // Nonce of the signer
        nonce,
        // Parcel signer pays 10 CCC as fee.
        fee: 10
    });
    const signedParcel = parcel.sign(signerSecret);
    // Send the signed parcel to the CodeChain node. The node will propagate this
    // parcel and attempt to confirm it.
    return sdk.sendSignedParcel(signedParcel);
}).then(parcelHash => {
    // sendSignedParcel returns a promise that resolves with a parcel hash if parcel has
    // been verified and queued successfully. It doesn't mean parcel was confirmed.
    console.log(`Parcel sent:`, parcelHash);
    return sdk.getParcel(parcelHash);
}).then((parcel) => {
    // getParcel returns a promise that resolves with a parcel.
    // blockNumber/blockHash/parcelIndex fields in Parcel is present only for the
    // confirmed parcel
    console.log(`Parcel`, parcel);
}).catch((err) => {
    console.error(`Error:`, err);
});
