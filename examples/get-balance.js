var SDK = require("codechain-sdk");

var sdk = new SDK({ server: "http://localhost:8080" });

sdk.rpc.chain.getBalance("cccqzn9jjm3j6qg69smd7cn0eup4w7z2yu9myd6c4d7").then(function (balance) {
    // the balance is a U256 instance at this moment.
    // Use toString() to print it out.
    console.log(balance.toString()); // the amount of CCC that the account has.
});
