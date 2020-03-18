# Transaction

Only a platform account can generate a transaction.
The transaction generator pays the transaction fees.
Transactions that cost less than the [minimum fee](Minimum-Fee.md) are rejected.
The minumum fee is different for each transaction type.

The seq must be identical with the payer’s account seq.
The account seq will be increased by 1 after a transaction is added to the block.
The amount of fee is deducted from the payer’s balance.
A transaction will not be included if the seq of the account doesn’t match or the balance of the account is less than the fee.

```rust
struct Transaction {
    seq: u64,
    fee: u64,
    network_id: NetworkId,
    action: Action,
}

enum Action {
    Pay { ..., },
    Custom { ..., },
}
```

### Timelock

A transaction fails if any `timelock` condition isn't met.
There are 4 types of `timelock`.
Basically, they keep the transaction from being executed until the specific point in time.
`Block` and `Time` types indicate the absolute time.
`BlockAge` and `TimeAge` types indicate relative time based on how long has the address been created.

- `Block(u64)`: The given value must be less than or equal to the current block's number.
- `BlockAge(u64)`: The given value must be less than or equal to the value `X`, where `X` = `current block number` - `the block number that the address was created at`.
- `Time(u64)`: The given value must be less than or equal to the current block's timestamp.
- `TimeAge(u64)`: The given value must be less than or equal to the value `X`, where `X` = `current block timestamp` - `the block timestamp that the address was created at`.

```rust
enum Timelock {
    Block(u64),
    BlockAge(u64),
    Time(u64),
    TimeAge(u64),
}
```

## Pay

`Pay` sends `quantity` amount of CCC to the `receiver`.

```rust
Pay {
    receiver: Address,
    quantity: u64,
}
```

## Custom

`Custom` is a special transaction.
The types of transactions that may exist depends on the consensus engine.

```rust
Custom {
    handler_id: u64,
    bytes: Bytes,
}
```
