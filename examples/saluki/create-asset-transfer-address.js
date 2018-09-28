const SDK = require("codechain-sdk");

const sdk = new SDK({
    server: "http://52.78.210.78:8080",
    networkId: "sc"
});

// LocalKeyStore creates `keystore.db` file in the working directory.
sdk.key
    .createLocalKeyStore()
    .then(function(keyStore) {
        return sdk.key.createAssetTransferAddress({
            keyStore
        });
    })
    .then(function(address) {
        // You can mint assets using this address.
        // Address example: scaqqqhzqech2kg7mmsf942xvn40cesxrgck0xxt9gpx6336xkeglwtdsqlg8lhr
        console.log(address.toString());
    });
