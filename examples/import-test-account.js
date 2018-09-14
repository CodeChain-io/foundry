var SDK = require("codechain-sdk");

var sdk = new SDK({
    server: process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080",
    networkId: process.env.CODECHAIN_NETWORK_ID || "tc"
});

var ACCOUNT_SECRET =
    process.env.ACCOUNT_SECRET ||
    "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd";
var ACCOUNT_PASSPHRASE = process.env.ACCOUNT_PASSPHRASE || "satoshi";

sdk.rpc.account
    .importRaw(ACCOUNT_SECRET, ACCOUNT_PASSPHRASE)
    .then(function(account) {
        console.log(account); // tccqzn9jjm3j6qg69smd7cn0eup4w7z2yu9my9a2k78
    })
    .catch(e => {
        if (e.message !== "Already Exists") {
            console.error(e);
        }
    });
