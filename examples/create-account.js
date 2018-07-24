var SDK = require("codechain-sdk");

var sdk = new SDK({ server: "http://localhost:8080" });

sdk.rpc.account.create("my-secret").then(function (account) {
    console.log(account); // 160-bit account id
});