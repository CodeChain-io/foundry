CodeChain SDK for JavaScript [![npm version](https://badge.fury.io/js/codechain-sdk.svg)](https://badge.fury.io/js/codechain-sdk) [![Build Status](https://travis-ci.org/CodeChain-io/codechain-sdk-js.svg?branch=master)](https://travis-ci.org/CodeChain-io/codechain-sdk-js) [![Gitter](https://badges.gitter.im/CodeChain-io/codechain-sdk-js.svg)](https://gitter.im/CodeChain-io/codechain-sdk-js?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge)
==============

A JavaScript SDK for CodeChain

## API Documentation

If you're looking for API documentation or quick start guide, [click here](https://api.codechain.io/)

## Features

 * Connect to a [CodeChain JSON-RPC](https://github.com/CodeChain-io/codechain/blob/master/spec/JSON-RPC.md) server from Node.js
 * Generate an account, create a transacton, sign a parcel.

## Getting Started

### Install

```
yarn install
```

### Running unit tests

Run `yarn test`

### Building and Running integration tests

1. Run `yarn build` command.
1. Run CodeChain RPC server.
1. Set `CODECHAIN_RPC_HTTP` environment variable with JSON-RPC HTTP server. (e.g. `http://localhost:8080`)
1. Run `yarn test-int` command.
   > It is also possible to indicate specific testcase with `-t` option. (e.g. `yarn test-int -t getBestBlockNumber`)

