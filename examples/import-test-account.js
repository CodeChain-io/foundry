var SDK = require("codechain-sdk");

var sdk = new SDK({ server: "http://localhost:8080" });

var secret = "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd";
var passphrase = "satoshi";
sdk.rpc.account.importRaw(secret, passphrase).then(function (account) {
    console.log(account); // 0xa6594b7196808d161b6fb137e781abbc251385d9
});
