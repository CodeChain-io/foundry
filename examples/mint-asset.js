var SDK = require("codechain-sdk");

var sdk = new SDK({ server: "http://localhost:8080" });

// If you want to know how to create an address, See the example "How to create
// an asset transfer address".
var address = "ccaqqqk7n0a0w69tjfza9svdjzhvu95cpl29ssnyn99ml8nvl8q6sd2c7qgjejfc";

var assetMintTransaction = sdk.core.createAssetMintTransaction({
    scheme: {
        shardId: 0,
        metadata: JSON.stringify({
            name: "Silver Coin",
            description: "...",
            icon_url: "...",
        }),
        amount: 100000000,
    },
    recipient: address,
});

// Send ChangeShardState type of the parcel to process the transaction.
var parcel = sdk.core.createChangeShardStateParcel({ transactions: [assetMintTransaction] });
sdk.rpc.chain.sendParcel(parcel, {
    account: "0xa6594b7196808d161b6fb137e781abbc251385d9",
    passphrase: "satoshi"
}).then(function (parcelHash) {
    // Get the invoice of the parcel.
    return sdk.rpc.chain.getParcelInvoice(parcelHash);
}).then(function (invoice) {
    // The invoice of ChangeShardState parcel is an array of the object that has
    // type { success: boolean }. Each object represents the result of each
    // transaction.
    console.log(invoice); // [{ success: true }]
});
