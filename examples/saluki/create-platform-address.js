var SDK = require("codechain-sdk");

var sdk = new SDK({
    server: "http://52.78.210.78:8080",
    networkId: "sc"
});

// LocalKeyStore creates `keystore.db` file in the working directory.
sdk.key
    .createLocalKeyStore()
    .then(function(keyStore) {
        return sdk.key.createPlatformAddress({
            keyStore
        });
    })
    .then(function(address) {
        // Visit https://saluki.codechain.io/faucet and get CCC for your account.
        console.log(address.toString());
    });
