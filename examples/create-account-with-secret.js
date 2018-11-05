const SDK = require("codechain-sdk");

const secret = SDK.util.generatePrivateKey();
console.log("Your secret:", secret);

const account = SDK.util.getAccountIdFromPrivate(secret);
const address = SDK.Core.classes.PlatformAddress.fromAccountId(account, {
    networkId: "tc"
});
console.log("Your CodeChain address:", address.toString());
