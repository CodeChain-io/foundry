var SDK = require("codechain-sdk");

var secret = SDK.util.generatePrivateKey();
console.log("Your secret:", secret);

var account = SDK.util.getAccountIdFromPrivate(secret);
var address = SDK.Core.classes.PlatformAddress.fromAccountId(account);
console.log("Your CodeChain address:", address.toString());
