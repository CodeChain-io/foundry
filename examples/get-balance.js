var SDK = require("codechain-sdk");

var sdk = new SDK({ server: "http://localhost:8080" });

sdk.rpc.chain.getBalance("0xa6594b7196808d161b6fb137e781abbc251385d9").then(function (balance) {
    // balance is U256 instance at this moment.
    // Use toString() to print.
    console.log(balance.toString()); // the amount of CCC that the account has.
});
