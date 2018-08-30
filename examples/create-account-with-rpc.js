var SDK = require("codechain-sdk");

var sdk = new SDK({ server: "http://localhost:8080" });

sdk.rpc.account
    .create("my-secret")
    .then(function(account) {
        console.log(account); // string that starts with "ccc"(mainnet) or "tcc"(testnet). For example: cccqzn9jjm3j6qg69smd7cn0eup4w7z2yu9myd6c4d7
    })
    .catch(console.error);
