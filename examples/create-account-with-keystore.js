var SDK = require("codechain-sdk");

var sdk = new SDK({ server: "http://localhost:8080" });

// createRemoteKeyStore("http://localhost:7007") is also available.
// If you want to know how to set up the external key store, go to
// https://codechain.readthedocs.io/en/latest/asset-management.html#use-remotekeystore-to-save-asset-address-private-key
sdk.key
    .createLocalKeyStore()
    .then(function(keyStore) {
        return sdk.key.createPlatformAddress({ keyStore });
    })
    .then(function(address) {
        console.log(address.toString());
    });
