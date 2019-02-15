CodeChain SDK Stakeholder Helper
==============

A JavaScript implementation for CodeChain stake token related custom actions and custom transactions

## Features

It adds the following features to [CodeChain SDK for JavaScript](https://github.com/CodeChain-io/codechain-sdk-js):

- Get the list of stakeholders
- Get the stake token balance of a stakeholder
- Send stake tokens

## How to

First, you need to install the package.

```sh
# npm
npm install codechain-sdk-stakeholder-helper

# yarn
yarn add codechain-sdk-stakeholder-helper
```

### Get the list of stakeholders
```js
const SDK = require("codechain-sdk");
const { getCCSHolders } = require("codechain-sdk-stakeholder-helper");

const sdk = new SDK({
  server: "http://localhost:8080",
  networkId: "tc"
});

getCCSHolders(sdk)
  .then(holders => {
    for(const holder of holders) {
      console.log(holder.toString());
    }
  });
```

### Get the stake token balance of a stakeholder
```js
const sdk = ...
const { getCCSBalance } = require("codechain-sdk-stakeholder-helper");

getCCSBalance("tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd")
  .then(balance => {
    console.log(balance.toString())
  })
```

### Send stake tokens
```js
const sdk = ...
const { createTransferCCSTransaction } = require("codechain-sdk-stakeholder-helper");

// Send 100 tokens to tccq94guhkrfndnehnca06dlkxcfuq0gdlamvw9ga4f
const tx = createTransferCCSTransaction(sdk, "tccq94guhkrfndnehnca06dlkxcfuq0gdlamvw9ga4f", 100);
const signedTx = tx.sign({ secret: "...", seq: "...", fee: "..." });
sdk.rpc.chain.sendSignedTransaction(signedTx)
  .then(txhash => {
    console.log(txhash.toString())
  });
```
