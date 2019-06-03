# CodeChain Stakeholder SDK

A JavaScript implementation for CodeChain stake token related custom actions and custom transactions

## Features

It adds the following features to [CodeChain SDK for JavaScript](https://github.com/CodeChain-io/codechain-sdk-js):

- Get the list of stakeholders
- Get the stake token balance of a stakeholder
- Get the list of delegations that a stakeholder delegated to delegatees
- Get the list of all pending revocations
- Transfer stake tokens
- Delegate stake tokens

## How to

You first need to install the package.

```sh
# npm
npm install codechain-stakeholder-sdk

# yarn
yarn add codechain-stakeholder-sdk
```

### Get the list of stakeholders

```js
const SDK = require("codechain-sdk");
const { getCCSHolders } = require("codechain-stakeholder-sdk");

const sdk = new SDK({
  server: "http://localhost:8080",
  networkId: "tc"
});

getCCSHolders(sdk)
  .then(holders => {
    // holders: PlatformAddress[]
    ...
  });
```

### Get the quantity of undelegated stake token of a stakeholder

```js
const sdk = ...
const { getUndelegatedCCS } = require("codechain-stakeholder-sdk");

getUndelegatedCCS(sdk, "tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd")
  .then(balance => {
    // balance: U64
    ...
  })
```

### Get the list of delegations that a stakeholder delegated to delegatees

```js
const sdk = ...
const { getDelegations } = require("codechain-stakeholder-sdk");

getDelegations(sdk, "tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd")
  .then(({ delegatee, quantity }) => {
    // delegatee: PlatformAddress
    // quantity: U64
    ...
  })
```

### Get the list of all pending revocations

```js
const sdk = ...
const { getPendingRevocations } = require("codechain-stakeholder-sdk");

getPendingRevocations(sdk)
  .then(({ delegator, delegatee, endTime, quantity }) => {
    // delegator: PlatformAddress
    // delegatee: PlatformAddress
    // endTime: number
    // quantity: U64
    ...
  })
```

### Transfer stake tokens

```js
const sdk = ...
const { createTransferCCSTransaction } = require("codechain-stakeholder-sdk");

// Transfer 100 tokens to tccq94guhkrfndnehnca06dlkxcfuq0gdlamvw9ga4f
const tx = createTransferCCSTransaction(sdk, "tccq94guhkrfndnehnca06dlkxcfuq0gdlamvw9ga4f", 100);
const signedTx = tx.sign({ secret: "...", seq: "...", fee: "..." });
sdk.rpc.chain.sendSignedTransaction(signedTx)
  .then(txhash => {
    // txhash: H256
    ...
  });
```

### Delegate stake tokens

```js
const sdk = ...
const { createDelegateCCSTransaction } = require("codechain-stakeholder-sdk");

// Delegate 100 tokens to tccq94guhkrfndnehnca06dlkxcfuq0gdlamvw9ga4f
const tx = createDelegateCCSTransaction(sdk, "tccq94guhkrfndnehnca06dlkxcfuq0gdlamvw9ga4f", 100);
const signedTx = tx.sign({ secret: "...", seq: "...", fee: "..." });
sdk.rpc.chain.sendSignedTransaction(signedTx)
  .then(txhash => {
    // txhash: H256
    ...
  });
```
