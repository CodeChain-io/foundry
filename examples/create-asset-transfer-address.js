var SDK = require("..");

var sdk = new SDK({ server: "http://localhost:8080" });

// MemoryKeyStore is a key store for testing purposes. Do not use this code in
// production.
var keyStore = sdk.key.createMemoryKeyStore();
// P2PKH supports P2PKH(Pay to Public Key Hash) lock/unlock scripts.
var p2pkh = sdk.key.createP2PKH({ keyStore });

p2pkh.createAddress().then(function (address) {
    // This type of address is used to receive assets when minting or transferring them.
    // Example: ccaqqqk7n0a0w69tjfza9svdjzhvu95cpl29ssnyn99ml8nvl8q6sd2c7qgjejfc
    console.log(address.toString());
}).catch(console.error);
