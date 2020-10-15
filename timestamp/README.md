# Timestamp Modules

This crate presents 5 modules and app-descriptor to construct a timestamp application.
It is the first set of modules ever implemented for Foundry,
and will keep evolving as it serves as an experimental stage of finding various patterns in writing modules.

Each module is designed to be as general as possible,
to see how the interface (represented as services) of such a reusable module would be.

We also have an app-descriptor for the application that loads all modules in this crate and constructs the working timestamp application.
Although it is just one possible instance of app-descriptor that composes the modules,
it will be the most appropriate and standard one.

The application constructed from such configuration is being used in various tests,
and you can understand how the modules exchange their services.

## Missing Components

There are some missing components.

- Transaction creation fields are not implemented for token/stamp module
- The format of final transaction body (module-specific content + public key + signature) is not properly exposed.

## Modules

### Account Module

Account module is for managing transaction sequences for accounts.
Most of transactions in blockchain have their signers, and each signer will have a sequence.
Sequence is an unsigned integer increased by one for every successful transactions, and is used to prevent replay of transactions.

All other modules defining their own transactions will use the account module to increase and check the sequences.
Account module itself defines a transaction as well, which is called `Hello`.
As you can notice by its name, its purpose is only for debugging and testing where using only the account module would be a convenient option.

### Token Module

Token module is for a general token managmenet system.
A token is identified with its issuer, and the user can freely give and take the ownership using transferring transaction.
However, issuing a new token can't be directly performed by the user.
Rather, such interface is opened as a service to the other modules, so that the module can issue tokens as a result of other transactions that it defines.
Other modules can query all tokens that a specific account owns, using the service.
It can also check the list of all accounts owning a specific token.

As you can see, token module manages existing token, but never defines the policy of issuing new tokens.
This makes the module general, adaptable, and so reusable.

### Stamp Module

Stamp module defines the stamping transaction.
While executing the transaction, it checkes whether the signer is valid in his sequence and token ownership.

You can specify the set of initial token owners as a genesis config.
It will ask token module to issue new tokens for the given accounts, at the genesis.
As explained in the section of token module, it can be freely transferred.

### Sorting Module

Sorting module is the implementor of `TxSorter`.
It sorts transactions based on their sequences,
using queyring services imported from each module.
Sorting module doesn't care about the content or even format of the given transaction,
but just retrieves the sequences by asking each module who defines such transaction.

### Staking Module

Staking module is the implementor of `InitConsensus` and `UpdateConsensus`.
It decides the validator set by the token distributed over accounts.

You can specify the set of initial token owners as a genesis config.
It will ask token module to issue new tokens for the given accounts, at the genesis.
As explained in the section of token module, it can be freely transferred.

Note that the token from stamp module is not related to staking module.
Each token is identified with its `issuer`, and will not be confused.
