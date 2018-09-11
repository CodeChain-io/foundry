var SDK = require("codechain-sdk");
var sdk = new SDK({
    server: "http://localhost:8080"
});

sdk.key
    .createAssetTransferAddress()
    .then(function(address) {
        // This type of address is used to receive assets when minting or transferring them.
        // Example: tcaqqq9pgkq69z488qlkvhkpcxcgfd3cqlkzgxyq9cewxuda8qqz7jtlvctt5eze
        console.log(address.toString());
    })
    .catch(console.error);
