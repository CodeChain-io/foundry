# codechain-sdk-js [![Build Status](https://travis-ci.com/kodebox-io/codechain-sdk-js.svg?token=ekWhXzYw9sUsATQJSpJ1&branch=master)](https://travis-ci.com/kodebox-io/codechain-sdk-js)

A JavaScript SDK for CodeChain

# Features

 * Connect to a [CodeChain JSON-RPC](https://github.com/kodebox-io/codechain/wiki/JSON-RPC) server from Node.js ~~or a web browser~~
 * Generate an account, create a transacton, sign a parcel.

# Getting Started

## Install

```
yarn install
```

## Run tests

### Unit tests

Run `yarn test`

### Build and Integration tests

1. Run `yarn build` command.
1. Run CodeChain RPC server.
1. Set `CODECHAIN_RPC_HTTP` environment variable with JSON-RPC HTTP server. (e.g. `https://localhost:8080`)
1. Run `yarn test-int` command.
   > It is also possible to indicate specific testcase with `-t` option. (e.g. `yarn test-int -t getBlockNumber`)


# Documentations

- [Basic Types](https://github.com/kodebox-io/codechain-sdk-js/wiki/Basic-Types)
- [API Specifications](https://github.com/kodebox-io/codechain-sdk-js/wiki/API-Specifications)
