var SDK = require("codechain-sdk");

var sdk = new SDK({
    server: process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080",
    networkId: process.env.CODECHAIN_NETWORK_ID || "tc"
});

var ACCOUNT_ADDRESS =
    process.env.ACCOUNT_ADDRESS ||
    "tccqzn9jjm3j6qg69smd7cn0eup4w7z2yu9my9a2k78";
var ACCOUNT_PASSPHRASE = process.env.ACCOUNT_PASSPHRASE || "satoshi";

// If you want to know how to create an address, see the example "Create an
// asset transfer address".
var address =
    "tcaqqq9pgkq69z488qlkvhkpcxcgfd3cqlkzgxyq9cewxuda8qqz7jtlvctt5eze";

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

// Send a change-shard-state parcel to process the transaction.
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
        // The invoice of change-shard-state parcel is an array of the object that has
        // type { success: boolean }. Each object represents the result of each
        // transaction.
        console.log(invoice); // [{ success: true }]
    })
    .catch(console.error);
