var SDK = require("codechain-sdk");

var sdk = new SDK({
    server: process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080",
    networkId: process.env.CODECHAIN_NETWORK_ID || "tc"
});

// LocalKeyStore creates `keystore.db` file in the working directory.
sdk.key
    .createLocalKeyStore()
    .then(function(keyStore) {
        return sdk.key.createAssetTransferAddress({
            type: "P2PKH", // It supports P2PKH(Pay to Public Key Hash) lock/unlock scripts.
            keyStore
        });
    })
    .then(function(address) {
        // This type of address is used to receive assets when minting or transferring them.
        // Example: tcaqqq9pgkq69z488qlkvhkpcxcgfd3cqlkzgxyq9cewxuda8qqz7jtlvctt5eze
        console.log(address.toString());
    });
