# Foundry Demo

## How to Run

1. Build Foundry
2. Place the binary in this directory
3. Run

```
RUST_LOG=warn ./foundry  --app-desc-path app-desc.toml --link-desc-path link-desc.toml --config config0.ini --db-path ./db0

RUST_LOG=warn ./foundry  --app-desc-path app-desc.toml --link-desc-path link-desc.toml --config config1.ini --db-path ./db1

RUST_LOG=warn ./foundry  --app-desc-path app-desc.toml --link-desc-path link-desc.toml --config config2.ini --db-path ./db2

RUST_LOG=warn ./foundry  --app-desc-path app-desc.toml --link-desc-path link-desc.toml --config config3.ini --db-path ./db3
```

for each node.

## Tutorial

Before start, please read these documents first.

- [Transaction](../spec/Transaction.md)
- [GraphQL](../spec/GraphQL.md)
- [Timestamp](../timestamp/README.md)

Here we provide a step-by-step tutorial to use timestamp application running on Foundry.
We will use only `Hello` transaction (which is defined by Account module).

### Required

Prepare a GraphQL client and the running network.
And also, build `codechain-keystore` to use command line program `sign`.
It takes private key and data as hex-encoded strings, and print the signature.

### Create an account

All modules in timestamp application are using Ed25519 as a transaction signing scheme.
Of course you can generate your own account, but in this tutorial, we use following pre-generated one.

- public: `e1e2fd66b0365c4c122f8084c128720285fcf0aa1b824cb749cbafe2151f9f99`
- private: `418a1804b82b366d2d88e348be571053f28f859fd550f97510fc52a9d08aacfce1e2fd66b0365c4c122f8084c128720285fcf0aa1b824cb749cbafe2151f9f99`

### Create a Transaction

The only parameter for creating a `Hello` transaction is the sequence of it.
Create a content of `Hello` transaction by executing following query to `/module-account/graphql`.

```
{
    txHello(seq: 0)
}
```

It will return

```
{
  "data": {
    "txHello": "a16373657100"
  }
}
```

Here, `a16373657100` is hex-encoded content of such transaction.

### Sign & Encode a Transaction

#### Using utill Module

We provide a special module called `util`.
It knows the signing scheme and format of transactions for timestamp application,
and provides a GraphQL field to actually sign and encode the transaction content.

Request following query to get the final transaction body, to `/module-util/graphql`:

```
{
  signAndEncodeTx(private: "418a1804b82b366d2d88e348be571053f28f859fd550f97510fc52a9d08aacfce1e2fd66b0365c4c122f8084c128720285fcf0aa1b824cb749cbafe2151f9f99", content:"a16373657100")
}
```

It returns

```
{
  "data": {
    "signAndEncodeTx": "a3697369676e61747572657882307863356232633736346131346634663638643832353034613362363630613038666537656563616161626662636232366334336134653966656262333133643039396266636264376466643335313061366333393239616266306662313065363039316462643339626434623231616163303162373063346137353861313130366d7369676e65725f7075626c6963784230786531653266643636623033363563346331323266383038346331323837323032383566636630616131623832346362373439636261666532313531663966393966616374696f6e8618a1186318731865187100"
  }
}
```

#### Using 3rd Parties Signing & Encoding Libraries

Use `sign` to make a signature for the transaction content.

```
./sign 418a1804b82b366d2d88e348be571053f28f859fd550f97510fc52a9d08aacfce1e2fd66b0365c4c122f8084c128720285fcf0aa1b824cb749cbafe2151f9f99 a16373657100
> c5b2c764a14f4f68d82504a3b660a08fe7eecaaabfbcb26c43a4e9febb313d099bfcbd7dfd3510a6c3929abf0fb10e6091dbd39bd4b21aac01b70c4a758a1106
```

To create a final `body` of a transaction, you should pack signature, public key and the content.

Encoding a transaction is still in a work-in-progress stage.
For now, you have to just use the specific format that the module uses.
Timestamp modules use CBOR with following field names - `signature`, `signer_public`, and `action`.

If you encode the transaction with the accounts given above, you will get

```
a3697369676e61747572657882307863356232633736346131346634663638643832353034613362363630613038666537656563616161626662636232366334336134653966656262333133643039396266636264376466643335313061366333393239616266306662313065363039316462643339626434623231616163303162373063346137353861313130366d7369676e65725f7075626c6963784230786531653266643636623033363563346331323266383038346331323837323032383566636630616131623832346362373439636261666532313531663966393966616374696f6e8618a1186318731865187100
```

### Send a Transaction

You can send your transaction to the node, using GraphQL mutation.
Note that you have to request this to to `/engine/graphql`.

```
mutation Mutation{
  sendTransaction(txType:"hello", body: "a3697369676e61747572657882307863356232633736346131346634663638643832353034613362363630613038666537656563616161626662636232366334336134653966656262333133643039396266636264376466643335313061366333393239616266306662313065363039316462643339626434623231616163303162373063346137353861313130366d7369676e65725f7075626c6963784230786531653266643636623033363563346331323266383038346331323837323032383566636630616131623832346362373439636261666532313531663966393966616374696f6e8618a1186318731865187100")
}
```

Will return

```
{
  "data": {
    "sendTransaction": "Done"
  }
}
```

### Check Execution of the Transaction

After you send a transaction, you can check whether it is included in the chain.
Send following request to `/module-account/graphql`.

```
{
  account(public:"e1e2fd66b0365c4c122f8084c128720285fcf0aa1b824cb749cbafe2151f9f99") {
    seq
  }
}
```

Will return

```
{
  "data": {
    "account": {
      "seq": 1
    }
  }
}
```

Wow! It accepted our transaction and excutued in chain, and ths sequence in state has been changed successfully.

### Query Blocks

You can also query blocks.

```
{
  block(number: 84) {
    header {
      number
    },
    transactions{
      txType
    }
  }
}
```

And it shows

```
{
  "data": {
    "block": {
      "header": {
        "number": 84
      },
      "transactions": [
        {
          "txType": "hello"
        }
      ]
    }
  }
}
```

Of course the number `84` where the transaction is included varies depending on time you send the tx.
