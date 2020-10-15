# Transaction

## Definition

Foundry defines a transaction as `txType` + `body`.
`txType` is the name of the transaction, as defined in the app-descriptor.
It is used only to route the transaction to the owning module.
`body` is an opaque content of the transaction.
Foundry host doesn't care about the value and the format is entirely defined by the module.

## Guideline

Signature of transaction is also part of the body.
Module can choose whatever signing scheme it wants, and even it can omit siganture for special transactions.
Thus creating a `body` from high-level parameters and user's private key is a module-specific process.
Foundry never manages this, and you should know how the module supports this.

However, there is a typical way that we encourage for modules.

1. Each module (who defines own transaction) exposes a GraphQL field to create an opaque **before-signed** transaction from high-level parameters.
2. Use trusted 3rd party program to sign the transaction.
3. Create a final `body`, which would be **before-signed** content + public key + signature.
This format is also module-specific, so the author could provide another GraphQL field or another stand-alone program.

2 and 3 might be merged into 1, so that user can get the final `body` right from the high-level parameters and his private key,
but it is not recommended unless he can 100% trust the module.

## How to Send

As explained [here](GraphQL.md), the engine itself has GraphQL endpoint too.
You can find `sendTransaction` field under the mutation root.
Pass `txType` and `body` using it, and it will be injected in the mempool.
