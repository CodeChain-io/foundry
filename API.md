## Table of Contents

- [Install package](#install-package)
- [Usage examples](#usage-examples)
  - [Setup the test account](#setup-the-test-account)
  - [Get the latest block number](#get-the-latest-block-number)
  - [Create a new account with a private key](#create-a-new-account-with-a-private-key)
  - [Create a new account with RPC](#create-a-new-account-with-rpc)
  - [Get the balance of an account](#get-the-balance-of-an-account)
  - [Send a payment transaction via sendTransaction](#send-a-payment-transaction-via-sendtransaction)
  - [Send a payment transaction via sendSignedTransaction](#send-a-payment-transaction-via-sendsignedtransaction)
  - [Create an asset transfer address](#create-an-asset-transfer-address)
  - [Mint a new asset](#mint-a-new-asset)
  - [Transfer assets](#transfer-assets)
- [SDK modules](#sdk-modules)

# Install package

```sh
# npm
npm install codechain-sdk
# yarn
yarn add codechain-sdk
```

# Usage examples

Make sure that your CodeChain RPC server is listening. In the examples, we assume that it is localhost:8080

## Setup the test account

Before you begin to meet various examples, you need to setup the account. The given account below(`tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd`) holds 100000 CCC at the genesis block. It's a sufficient quantity to pay for the transaction fee.

```javascript
var SDK = require("codechain-sdk");
var sdk = new SDK({ server: "http://localhost:8080" });

var secret = "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd";
var passphrase = "satoshi";
sdk.rpc.account.importRaw(secret, passphrase).then(function(account) {
  console.log(account); // tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd
});
```

---

## Get the latest block number

You can retrieve the chain information using methods in `sdk.rpc.chain`.

```javascript
var SDK = require("codechain-sdk");
var sdk = new SDK({ server: "http://localhost:8080" });

sdk.rpc.chain.getBestBlockNumber().then(function(num) {
  console.log(num);
});
```

---

## Create a new account with a private key

```javascript
var SDK = require("codechain-sdk");

var secret = SDK.util.generatePrivateKey();
console.log("Your secret:", secret);

var account = SDK.util.getAccountIdFromPrivate(secret);
var address = SDK.Core.classes.PlatformAddress.fromAccountId(account, {
  networkId: "tc"
});
console.log("Your CodeChain address:", address.toString());
```

---

## Create a new account with RPC

You can manage accounts and create their signatures using methods in `sdk.rpc.account`.

```javascript
var SDK = require("codechain-sdk");
var sdk = new SDK({ server: "http://localhost:8080" });

var passphrase = "my-secret";
sdk.rpc.account.create(passphrase).then(function(account) {
  console.log(account); // string that starts with either "tcc"(Solo testnet) or "ccc"(mainnet). For example: tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd
});
```

---

## Get the balance of an account

You can get the balance of an account using `getBalance` method in `sdk.rpc.chain`. See also `getSeq`, `getRegularKey`.

```javascript
var SDK = require("codechain-sdk");
var sdk = new SDK({ server: "http://localhost:8080" });

sdk.rpc.chain
  .getBalance("tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd")
  .then(function(balance) {
    // the balance is a U256 instance at this moment. Use toString() to print it out.
    console.log(balance.toString()); // the quantity of CCC that the account has.
  });
```

---

## Send a payment transaction via sendTransaction

When you create an account, the CCC balance is 0. CCC is needed to pay for the transaction's fee. The fee must be at least 10 for any transaction. The example below shows the sending of 10000 CCC from the test account(`tccqzn..9a2k78`) to the account(`tccqru..7vzngg`).

```javascript
var SDK = require("codechain-sdk");
var sdk = new SDK({ server: "http://localhost:8080" });

var tx = sdk.core.createPayTransaction({
  recipient: "tccqxv9y4cw0jwphhu65tn4605wadyd2sxu5yezqghw",
  quantity: 10000
});

sdk.rpc.chain
  .sendTransaction(tx, {
    account: "tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd",
    passphrase: "satoshi"
  })
  .then(function(hash) {
    return sdk.rpc.chain.getTransactionResult(hash, { timeout: 300 * 1000 });
  })
  .then(function(result) {
    console.log(result); // true
  });
```

---

## Send a payment transaction via sendSignedTransaction

```javascript
var SDK = require("codechain-sdk");

var sdk = new SDK({ server: "http://localhost:8080" });

var tx = sdk.core.createPayTransaction({
  recipient: "tccqxv9y4cw0jwphhu65tn4605wadyd2sxu5yezqghw",
  quantity: 10000
});

var account = "tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd";
var accountSecret =
  "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd";

sdk.rpc.chain
  .getSeq(account)
  .then(function(seq) {
    return sdk.rpc.chain.sendSignedTransaction(
      tx.sign({
        secret: accountSecret,
        fee: 10,
        seq: seq
      })
    );
  })
  .then(function(hash) {
    return sdk.rpc.chain.getTransactionResult(hash, { timeout: 300 * 1000 });
  })
  .then(function(result) {
    console.log(result); // true
  });
```

---

## Create an asset transfer address

```javascript
var SDK = require("codechain-sdk");
var sdk = new SDK({
  server: "http://localhost:8080"
});

sdk.key
  .createAssetTransferAddress()
  .then(function(address) {
    // This type of address is used to receive assets when minting or transferring them.
    // Example: tcaqqq9pgkq69z488qlkvhkpcxcgfd3cqlkzgxyq9cewxuda8qqz7jtlvctt5eze
    console.log(address.toString());
  })
  .catch(console.error);
```

---

## Mint a new asset

```javascript
var SDK = require("codechain-sdk");
var sdk = new SDK({ server: "http://localhost:8080" });

async function mintNewAsset() {
  var address = await sdk.key.createAssetTransferAddress()
  var tx = sdk.core.createMintAssetTransaction({
    scheme: {
      shardId: 0,
      metadata: JSON.stringify({
        name: "Silver Coin",
        description: "...",
        icon_url: "..."
      }),
      supply: 100000000
    },
    recipient: address
  });

  sdk.rpc.chain
    .sendTransaction(tx, {
      account: "tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd",
      passphrase: "satoshi"
    })
    .then(function(hash) {
      // Get the result of the transaction.
      return sdk.rpc.chain.getTransactionResult(hash, {
        // Wait up to 120 seconds to get the result.
        timeout: 120 * 1000
      });
    })
    .then(function(result) {
      // The result of the mint transaction is a boolean.
      console.log(result); // true
    });
}
mintNewAsset();
```

---

## Transfer assets

The entire example can be viewed [here](https://github.com/CodeChain-io/codechain-sdk-js/blob/master/examples/mint-and-transfer.js).

---

# SDK modules

- [RPC](classes/rpc.html)
  - [node](classes/noderpc.html)
  - [chain](classes/chainrpc.html)
  - [network](classes/networkrpc.html)
  - [account](classes/accountrpc.html)
  - [engine](classes/enginerpc.html)
- [Core](classes/core.html)
  - [classes](classes/core.html#classes-1) (Block, Transaction, ...)
- [Utility](classes/sdk.html#util)
