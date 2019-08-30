# CodeChain Stakeholder CLI: ccstake

CLI tools for CodeChain stakeholders, and validators.

## Features

- CCS: [show](#show), [transfer](#transfer), [delegate](#delegate), [revoke](#revoke), [redelegate](#revoke), [batch-delegate](#batch-delegate)
- Governance: [sign](#sign), [change-params](#change-params)
- Validator: [validators](#validators), [self-nominate](#validators)

## Usages

You can see the actual usages with more context here.
* [CodeChain User Guide: Staking](https://codechain.readthedocs.io/en/latest/user-guide/staking/)
* [CodeChain User Guide: Running a Validator Node](https://codechain.readthedocs.io/en/latest/user-guide/running-a-validator-node/)

### Installation

```bash
yarn global add codechain-stakeholder-cli
```

You also need cckey to manage the local keystore database: https://github.com/codechain-io/codechain-keystore-cli

If you successfully installed `ccstake`, you can see the overview with
```bash
$> ccstake
ccstake <command>

Commands:
  ccstake show [account]                    Show staking status of an account
  ccstake transfer                          Transfer CCS to an account
  ccstake delegate                          Delegate CCS to an account
  ccstake batch-delegate                    Batch manage delegations through
  <distribution-file>                       distribution file
  ccstake revoke                            Revoke delegation to an account
  ccstake redelegate                        Move a delegation to another account
  ccstake self-nominate                     Self nominate as a candidate
  ccstake validators                        Show validators
  ccstake sign                              Sign a message
  ccstake change-params                     Change CodeChain network parameter

Common:
  --version     Show version number                                    [boolean]
  --keys-path   The path to storing the keys [string] [default: "./keystore.db"]
  --rpc-server  The RPC server URL
                                 [string] [default: "https://rpc.codechain.io/"]
  --help        Show help                                              [boolean]

Not enough non-option arguments: got 0, need at least 1
```

### Managing CCS

#### show

To see overview of CCS distribution, you can use `show` command.

```bash
$> ccstake show
```

You can see a specific account's CCS state by passing an address to the `show` command.

```bash
$> ccstake show cccqxyyc4yu3pc2pzl2y0tec26qxau3a27lq5ntee9j
```

#### transfer

To transfer CCS to someone from an account, you can use the `transfer` command.

```bash
$> ccstake transfer \
     --account cccqxyyc4yu3pc2pzl2y0tec26qxau3a27lq5ntee9j \
     --recipient cccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6y4u3qm5 \
     --quantity 10000 \
     --fee 10
```

#### delegate

To delegate CCS to someone from an account, you can use the `delegate` command.

```bash
$> ccstake delegate \
     --account cccqxyyc4yu3pc2pzl2y0tec26qxau3a27lq5ntee9j \
     --delegatee cccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6y4u3qm5 \
     --quantity 10000 \
     --fee 10
```

#### revoke

To revoke delegated CCS from a delegatee to an account, you can use the `revoke` command.

```bash
$> ccstake revoke \
     --account cccqxyyc4yu3pc2pzl2y0tec26qxau3a27lq5ntee9j \
     --delegatee cccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6y4u3qm5 \
     --quantity 10000 \
     --fee 10
```

#### Redelegate

To move a delegation from an existing delegatee to another delegatee, you can use the `redelegate` command.

```bash
$> ccstake redelegate \
     --account cccqxyyc4yu3pc2pzl2y0tec26qxau3a27lq5ntee9j \
     --previous-delegatee cccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6y4u3qm5 \
     --next-delegatee cccq9qwg08jnn4agnaex3pty5hcq04m2h87ryxh9p5q \
     --quantity 10000 \
     --fee 10
```

#### batch-delegate

To manage multiple delegations to validators across multiple stakeholder accounts, you can use the `batch-delegate` command. To use it, you need a distribution file, and a password file.

The distribution file is a json file similar to this:
```json
{
  "accounts": [
    "cccq9qwg08jnn4agnaex3pty5hcq04m2h87ryxh9p5q",
    "cccqyuzgh3y8w3xtrzdfrjs6yk6wrhh30y6gys2nv6l",
  ],
  "fee": 10,
  "distributions": [
    {
      "delegatee": "cccq98jmz9muznaun3xhtmumt7txx8d4ehdlcn5v3hz",
      "quantity": 10000
    },
    {
      "delegatee": "cccqyyk336h4d5ddv20h6hhdh35u6r7j5dn7chl2xaz",
      "quantity": 20000
    },
    {
      "delegatee": "cccq8hekjzqhpcha528jalj2qyjhd5849kpxgrhfc76",
      "quantity": 30000
    },
  ]
}
```

The password file is similar to this:
```json
[
    { "address": "cccq9qwg08jnn4agnaex3pty5hcq04m2h87ryxh9p5q", "password": "super-strong-password" },
    { "address": "cccqyuzgh3y8w3xtrzdfrjs6yk6wrhh30y6gys2nv6l", "password": "very-strong" }
]
```
The password file should contain passwords of all accounts in the distribution file.

When these file is prepared, you can `--dry-run` to see if the planned transactions distribute accordingly.

```bash
$> ccstake batch-delegate ./distribution.json \
     --password-path=./passwords.json \
     --dry-run
```

If you are satisfied with the plan, you can go ahead with the following command:

```bash
$> ccstake batch-delegate ./distribution.json \
     --password-path=./passwords.json
```

However if there are changes in the overall situation, the plan can be changed from the plan in `--dry-run`, and even the execution might fail.
Since the execution of transactions are not atomic, successful transactions are not reverted and the state if some of the transactions in the plan fail.

### Governance

In the CodeChain, stakeholder governance is reflected by changing the parameters of the chain.

You can prepare the new parameter with this tool: https://codechain-io.github.io/codechain-change-common-params/

When the new parameters are prepared, you can sign it and send it to the chain with commands below.

#### change-params

When you've collected enough signatures, and the transaction is ready to be sent, you can send it to the chain with the following command:

```bash
$> ccstake change-params \
     --transaction <prepared transaction here> \
     --account cccq9qwg08jnn4agnaex3pty5hcq04m2h87ryxh9p5q \
     --fee 10
```

#### sign

When someone proposes new parameters to change and it seems good, you can sign the parmeter to show agreement with the command below.
If the message is successfully signed, you can send it to the proposer manually.

```bash
$> ccstake sign \
     --account cccq9qwg08jnn4agnaex3pty5hcq04m2h87ryxh9p5q \
     --message <proposed parameters> \
```

### Validator

#### validators

You can query the overall status of dynamic validators.
You can see the list of candidates, validators, jailed accounts, banned accounts with this command:

```bash
$> ccstake validators
```

#### self-nominate

When you are ready to become a valiator, you can self-nominate with this command:

```bash
$> ccstake self-nominate \
     --account cccq9qwg08jnn4agnaex3pty5hcq04m2h87ryxh9p5q \
     --deposit 10000000 \
     --metadata "CodeChain validator <http://codechain.io/>" \
     --fee 10
```
