# Table of Contents

1. [Install package](#install-package)
1. [Usage examples](#usage-examples)
    1. [Setup the test account](#setup-the-test-account)
    1. [Get the latest block number](#get-the-latest-block-number)
    1. [Create a new account](#create-a-new-account)
    1. [Get the balance of an account](#get-the-balance-of-an-account)
    1. [Send a payment parcel](#send-a-payment-parcel)
    1. [Create an asset transfer address](#create-an-asset-transfer-address)
    1. [Mint a new asset](#mint-a-new-asset)

1. [SDK modules](#sdk-modules)

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

Before you begin to meet various examples, you need to setup the account. The given account below(`0xa6594b7196808d161b6fb137e781abbc251385d9`) holds 100000 CCC at the genesis block. It's a sufficient amount to pay for the parcel fee.

```javascript
var SDK = require("codechain-sdk");
var sdk = new SDK({ server: "http://localhost:8080" });

var secret = "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd";
var passphrase = "satoshi";
sdk.rpc.account.importRaw(secret, passphrase).then(function (account) {
    console.log(account); // 0xa6594b7196808d161b6fb137e781abbc251385d9
});
```

---

## Get the latest block number

You can retrieve the chain information using methods in `sdk.rpc.chain`.

```javascript
var SDK = require("codechain-sdk");
var sdk = new SDK({ server: "http://localhost:8080" });

sdk.rpc.chain.getBestBlockNumber().then(function (num) {
    console.log(num);
});
```

---

## Create a new account

You can manage accounts and create their signatures using methods in `sdk.rpc.account`.

```javascript
var SDK = require("codechain-sdk");
var sdk = new SDK({ server: "http://localhost:8080" });

var passphrase = "my-secret";
sdk.rpc.account.create(passphrase).then(function (account) {
    console.log(account); // 160-bit account id
});
```

---

## Get the balance of an account

You can get the balance of an account using `getBalance` method in `sdk.rpc.chain`. See also `getNonce`, `getRegularKey`.

```javascript
var SDK = require("codechain-sdk");
var sdk = new SDK({ server: "http://localhost:8080" });

sdk.rpc.chain.getBalance("0xa6594b7196808d161b6fb137e781abbc251385d9").then(function (balance) {
    // the balance is a U256 instance at this moment. Use toString() to print it out.
    console.log(balance.toString()); // the amount of CCC that the account has.
});
```

---

## Send a payment parcel

When you create an account, the CCC balance is 0. CCC is needed to pay for the parcel's fee. The fee must be at least 10 for any parcel. The example below shows the sending of 10000 CCC from the test account(`0xa659..85d9`) to the account(`0xaaaa..aaaa`).

```javascript
var SDK = require("codechain-sdk");
var sdk = new SDK({ server: "http://localhost:8080" });

var parcel = sdk.core.createPaymentParcel({
    recipient: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    amount: 10000
});

sdk.rpc.chain.sendParcel(parcel, {
    account: "0xa6594b7196808d161b6fb137e781abbc251385d9",
    passphrase: "satoshi"
}).then(function (parcelHash) {
    return sdk.rpc.chain.getParcelInvoice(parcelHash, { timeout: 5 * 60 * 1000 });
}).then(function (parcelInvoice) {
    console.log(parcelInvoice) // { success: true }
});
```

---

## Create an asset transfer address

```javascript
var SDK = require("..");
var sdk = new SDK({ server: "http://localhost:8080" });

// MemoryKeyStore is a key store for testing purposes. Do not use this code in
// production.
var keyStore = sdk.key.createMemoryKeyStore();
// P2PKH supports P2PKH(Pay to Public Key Hash) lock/unlock scripts.
var p2pkh = sdk.key.createP2PKH({ keyStore });

p2pkh.createAddress().then(function (address) {
    // This type of address is used to receive assets when minting or transferring them.
    // Example: ccaqqqk7n0a0w69tjfza9svdjzhvu95cpl29ssnyn99ml8nvl8q6sd2c7qgjejfc
    console.log(address.toString());
});
```

---

## Mint a new asset

```javascript
var SDK = require("codechain-sdk");
var sdk = new SDK({ server: "http://localhost:8080" });

// If you want to know how to create an address, See the example "Create an
// asset transfer address".
var address = "ccaqqqk7n0a0w69tjfza9svdjzhvu95cpl29ssnyn99ml8nvl8q6sd2c7qgjejfc";

var assetMintTransaction = sdk.core.createAssetMintTransaction({
    scheme: {
        shardId: 0,
        metadata: JSON.stringify({
            name: "Silver Coin",
            description: "...",
            icon_url: "...",
        }),
        amount: 100000000,
    },
    recipient: address,
});

// Send a change-shard-state parcel to process the transaction.
var parcel = sdk.core.createChangeShardStateParcel({ transactions: [assetMintTransaction] });
sdk.rpc.chain.sendParcel(parcel, {
    account: "0xa6594b7196808d161b6fb137e781abbc251385d9",
    passphrase: "satoshi"
}).then(function (parcelHash) {
    // Get the invoice of the parcel.
    return sdk.rpc.chain.getParcelInvoice(parcelHash, {
        // Wait up to 120 seconds to get the invoice
        timeout: 120 * 1000
    });
}).then(function (invoice) {
    // The invoice of ChangeShardState parcel is an array of the object that has
    // type { success: boolean }. Each object represents the result of each
    // transaction.
    console.log(invoice); // [{ success: true }]
});
```

---

## Transfer assets

The brief version of example will be appeared soon. The entire example can be viewed [here](https://github.com/CodeChain-io/codechain-sdk-js/blob/master/examples/mint-and-transfer.js).

---

# SDK modules

 * [RPC](classes/rpc.html)
   * [node](classes/noderpc.html)
   * [chain](classes/chainrpc.html)
   * [network](classes/networkrpc.html)
   * [account](classes/accountrpc.html)
 * [Core](classes/core.html)
   * [classes](classes/core.html#classes-1) (Block, Parcel, Transaction, ...)
 * [Utility](classes/sdk.html#util)
