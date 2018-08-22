var SDK = require("codechain-sdk");

var sdk = new SDK({ server: "http://localhost:8080" });

// createLocalKeyStore() is also available.
sdk.key.createRemoteKeyStore("http://localhost:7007")
    .then(function (keyStore) {
        return sdk.key.createPlatformAddress({ keyStore });
    })
    .then(function (address) {
        console.log(address.toString());
    });