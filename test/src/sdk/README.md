CodeChain SDK for JavaScript [![npm version](https://badge.fury.io/js/codechain-sdk.svg)](https://badge.fury.io/js/codechain-sdk) [![Build Status](https://travis-ci.org/CodeChain-io/codechain-sdk-js.svg?branch=master)](https://travis-ci.org/CodeChain-io/codechain-sdk-js) [![Gitter](https://badges.gitter.im/CodeChain-io/codechain-sdk-js.svg)](https://gitter.im/CodeChain-io/codechain-sdk-js?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge)
==============

A JavaScript SDK for CodeChain

## API Documentation (For SDK users)

If you're looking for an API documentation or a quick start guide, [click here](https://api.codechain.io/)

## Features

 * Connect to a [CodeChain JSON-RPC](https://github.com/CodeChain-io/codechain/blob/master/spec/JSON-RPC.md) server
 * Generate an account, create a transacton, sign a transaction

## Getting Started (For SDK developers)

### Clone the source code

```
git clone git@github.com:CodeChain-io/codechain-sdk-js.git
```

### Install dependencies

```
cd codechain-sdk-js && npm install
```

### Running unit tests

```
npm run test
```

### Building and running integration tests

1. Run `yarn build` command.
1. Run CodeChain RPC server. 
1. Run `yarn test-int` command.
   > It is also possible to indicate specific testcases with --`testRegex` and `-t` option. (e.g. `yarn test-int --testRegex Rpc -t getBestBlockNumber`)

