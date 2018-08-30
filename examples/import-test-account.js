var SDK = require("codechain-sdk");

var sdk = new SDK({ server: "http://localhost:8080" });

var secret = "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd";
var passphrase = "satoshi";
sdk.rpc.account
    .importRaw(secret, passphrase)
    .then(function(account) {
        console.log(account); // tccqzn9jjm3j6qg69smd7cn0eup4w7z2yu9my9a2k78
    })
    .catch(e => {
        if (e.message !== "Already Exists") {
            console.error(e);
        }
    });
