# foundry-primitives-js [![Build Status](https://travis-ci.org/CodeChain-io/foundry-primitives-js.svg?branch=master)](https://travis-ci.org/CodeChain-io/foundry-primitives-js)

JavaScript functions and classes for Foundry primitives

## Installing a package

```sh
# npm
npm install foundry-primitives
# yarn
yarn add foundry-primitives
```

## Getting started

```javascript
// Using require
var primitives = require("foundry-primitives");
var H256 = primitives.H256;
var blake256 = primitives.blake256;

// Using import
import { blake256, H256 } from "foundry-primitives";
```

## Functions

- blake256
- blake256WithKey
- ripemd160
- signEd25519
- verifyEd25519
- generatePrivateKey
- getPublicFromPrivate
- toHex
- toArray
- getAccountIdFromPrivate
- getAccountIdFromPublic

## Classes

- H128, H160, H256, H512
- U64, U128, U256
- Address

## API Documentation

[https://codechain-io.github.io/foundry-primitives-js](https://codechain-io.github.io/foundry-primitives-js)
