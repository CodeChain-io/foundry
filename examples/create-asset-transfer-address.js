var SDK = require("codechain-sdk");
var sdk = new SDK({ server: "http://localhost:8080" });

// createRemoteKeyStore("http://localhost:7007") is also available.
// If you want to know how to set up the external key store, go to
// https://codechain.readthedocs.io/en/latest/asset-management.html#use-remotekeystore-to-save-asset-address-private-key
sdk.key
    .createLocalKeyStore()
    .then(function(keyStore) {
        // P2PKH supports P2PKH(Pay to Public Key Hash) lock/unlock scripts.
        var p2pkh = sdk.key.createP2PKH({ keyStore });
        return p2pkh.createAddress();
    })
    .then(function(address) {
        // This type of address is used to receive assets when minting or transferring them.
        // Example: tcaqqq9pgkq69z488qlkvhkpcxcgfd3cqlkzgxyq9cewxuda8qqz7jtlvctt5eze
        console.log(address.toString());
    })
    .catch(console.error);
