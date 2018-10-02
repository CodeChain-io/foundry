var SDK = require("codechain-sdk");

var sdk = new SDK({
    server: process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080",
    networkId: process.env.CODECHAIN_NETWORK_ID || "tc"
});

var ACCOUNT_ADDRESS =
    process.env.ACCOUNT_ADDRESS ||
    "tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd";
var ACCOUNT_PASSPHRASE = process.env.ACCOUNT_PASSPHRASE || "satoshi";

// If you want to know how to create an address, see the example "Create an
// asset transfer address".
var address =
    "tcaqqqeqq47kh5nlk67eua0w2k4tku2hm0hazqx5wa3eqaaslq7zdfxhwgxs0x2r";

var assetMintTransaction = sdk.core.createAssetMintTransaction({
    scheme: {
        shardId: 0,
        worldId: 0,
        metadata: JSON.stringify({
            name: "Silver Coin",
            description: "...",
            icon_url: "..."
        }),
        amount: 100000000
    },
    recipient: address,
    nonce: Math.floor(Math.random() * 1000000000)
});

// Send an asset-transaction-group parcel to process the transaction.
var parcel = sdk.core.createAssetTransactionGroupParcel({
    transactions: [assetMintTransaction]
});
sdk.rpc.chain
    .sendParcel(parcel, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    })
    .then(function(parcelHash) {
        // Get the invoice of the parcel.
        return sdk.rpc.chain.getParcelInvoice(parcelHash, {
            // Wait up to 120 seconds to get the invoice.
            timeout: 120 * 1000
        });
    })
    .then(function(invoice) {
        // The invoice of asset-transaction-group parcel is an array of the object that has
        // type { success: boolean }. Each object represents the result of each
        // transaction.
        console.log(invoice); // [{ success: true }]
    })
    .catch(console.error);
