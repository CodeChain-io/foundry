var SDK = require("..");

var sdk = new SDK({ server: "http://localhost:8080" });

// MemoryKeyStore is a key store for testing purposes. Do not use this code in
// production.
var keyStore = sdk.key.createMemoryKeyStore();
// P2PKH supports P2PKH(Pay to Public Key Hash) lock/unlock scripts.
var p2pkh = sdk.key.createP2PKH({ keyStore });

p2pkh.createAddress().then(function (address) {
    // This type of address is used to receive assets when minting or transferring them.
    // Example: tcaqqq9pgkq69z488qlkvhkpcxcgfd3cqlkzgxyq9cewxuda8qqz7jtlvctt5eze
    console.log(address.toString());
}).catch(console.error);
