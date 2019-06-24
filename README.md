# CodeChain Stakeholder SDK

A JavaScript implementation for CodeChain stake token related custom actions and custom transactions

## Features

It adds the following features to [CodeChain SDK for JavaScript](https://github.com/CodeChain-io/codechain-sdk-js):

- Query staking states
- Call staking related RPC
- Create staking transactions

## How to

You first need to install the package.

```sh
# npm
npm install codechain-stakeholder-sdk

# yarn
yarn add codechain-stakeholder-sdk
```

Then prepare SDK instance as usual.

```js
import { SDK } from "codechain-sdk";
const sdk = new SDK({
  server: "http://localhost:8080",
  networkId: "tc"
});
```

Now, you are prepared to use `stakeholder-sdk-js`

### Query staking states

These functions can have an optional block number parameter at the end.

#### Get the list of stakeholders

```js
import { getCCSHolders } from "codechain-stakeholder-sdk";

const holders = await getCCSHolders(sdk);
// holders: PlatformAddress[]
```

#### Get the quantity of undelegated stake token of a stakeholder

```js
import { getUndelegatedCCS } from "codechain-stakeholder-sdk";

const balance = await getUndelegatedCCS(
  sdk,
  "tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd"
);
// balance: U64
```

#### Get the list of delegations that a stakeholder delegated to delegatees

```js
import { getDelegations } from "codechain-stakeholder-sdk";

const delegations = await getDelegations(
  sdk,
  "tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd"
);
for (const { delegatee, quantity } of delegations) {
  // delegatee: PlatformAddress
  // quantity: U64
}
```

#### Get the list of validator candidates

```js
import { getCandidates } from "codechain-stakeholder-sdk";

const candidates = await getCandidates(sdk);
for (const { pubkey, deposit, nominationEndsAt, metadata } of candidates) {
  // pubkey: H512
  // deposit: U64
  // nominationEndsAt: U64
  // metadata: Buffer
}
```

#### Get the list of jailed accounts

```js
import { getJailed } from "codechain-stakeholder-sdk";

const prisoners = await getJailed(sdk);
for (const { address, deposit, custodyUntil, releasedAt } of prisoners) {
  // address: PlatformAddress
  // deposit: U64
  // custodyUntil: U64
  // releasedAt: U64
}
```

#### Get the list of banned accounts

```js
import { getBanned } from "codechain-stakeholder-sdk";

const banned = await getBanned(sdk);
// banned: PlatformAddress[]
```

#### Get intermediate rewards

```js
import { getIntermediateRewards } from "codechain-stakeholder-sdk";

const { previous, current } = await getIntermediateRewards(sdk);
// previous, current: { address: PlatformAddress, quantity: U64 }[]
```

#### Get the list of current validators

```js
import { getValidators } from "codechain-stakeholder-sdk";

const validators = await getValidators(sdk);
for (const { weight, delegation, deposit, pubkey } of validators) {
  // weight: U64
  // delegation: U64
  // deposit: U64
  // pubkey: H512
}
```

### RPCs to query staking status

#### TermMetadata

```js
import { getTermMetadata } from "codechain-stakeholder-sdk";

const { lastTermFinishedBlockNumber, currentTermId } = await getTermMetadata(
  sdk
);
```

### Create staking transactions

#### Transfer stake tokens

```js
import { createTransferCCSTransaction } from "codechain-stakeholder-sdk";

// Transfer 100 tokens to tccq94guhkrfndnehnca06dlkxcfuq0gdlamvw9ga4f
const tx = createTransferCCSTransaction(
  sdk,
  "tccq94guhkrfndnehnca06dlkxcfuq0gdlamvw9ga4f",
  100
)
const signedTx = .sign({ secret: "...", seq: "...", fee: "..." });
const txhash = await sdk.rpc.chain.sendSignedTransaction(signedTx);
```

#### Delegate stake tokens

```js
import { createDelegateCCSTransaction } from "codechain-stakeholder-sdk";

// Delegate 100 tokens to tccq94guhkrfndnehnca06dlkxcfuq0gdlamvw9ga4f
const tx = createDelegateCCSTransaction(
  sdk,
  "tccq94guhkrfndnehnca06dlkxcfuq0gdlamvw9ga4f",
  100
);
const signedTx = tx.sign({ secret: "...", seq: "...", fee: "..." });
const txhash = await sdk.rpc.chain.sendSignedTransaction(signedTx);
```

#### Revoke stake tokens

```js
import { createRevokeTransaction } from "codechain-stakeholder-sdk";

// Revoke 100 tokens delegated to tccq94guhkrfndnehnca06dlkxcfuq0gdlamvw9ga4f
const tx = createRevokeTransaction(
  sdk,
  "tccq94guhkrfndnehnca06dlkxcfuq0gdlamvw9ga4f",
  100
);
const signedTx = tx.sign({ secret: "...", seq: "...", fee: "..." });
const txhash = await sdk.rpc.chain.sendSignedTransaction(signedTx);
```

#### Self-nominate

```js
import { createSelfNominateTransaction } from "codechain-stakeholder-sdk";

// Self-nominate with 1000 CCC and metadata
const tx = createSelfNominateTransaction(sdk, 1000, "some-metadata");
const signedTx = tx.sign({ secret: "...", seq: "...", fee: "..." });
const txhash = await sdk.rpc.chain.sendSignedTransaction(signedTx);
```
