import { SDK } from "codechain-sdk";
import { PlatformAddress, U64 } from "codechain-sdk/lib/core/classes";
import { createDelegateCCSTransaction } from "codechain-stakeholder-sdk";
import * as yargs from "yargs";

import { GlobalParams } from "..";
import { summarize } from "../summerizer";
import {
    askPasspharaseFor,
    asyncHandler,
    coerce,
    minusChangeArgs,
    plusChangeArgs,
    prologue,
    waitForTx
} from "../util";

interface DelegateParams extends GlobalParams {
    account: PlatformAddress;
    delegatee: PlatformAddress;
    quantity: U64;
    fee: number;
}

export const module: yargs.CommandModule<GlobalParams, DelegateParams> = {
    command: "delegate",
    describe: "Delegate CCS to an account",
    builder(args) {
        return args
            .option("account", {
                coerce: coerce("account", PlatformAddress.ensure),
                demand: true
            })
            .option("delegatee", {
                coerce: coerce("delegatee", PlatformAddress.ensure),
                demand: true
            })
            .option("quantity", {
                coerce: coerce("quantity", U64.ensure),
                demand: true
            })
            .option("fee", {
                number: true,
                demand: true
            });
    },
    handler: asyncHandler(async argv => {
        const { sdk, blockNumber } = await prologue(argv);

        console.log("=== Confirm your action ===");
        console.log("Action:", "Delegate");
        console.log("Quantity:", argv.quantity.toLocaleString());
        await printSummary(sdk, blockNumber, argv.account, argv.delegatee, {
            ccsChanges: argv.quantity,
            cccChanges: U64.ensure(argv.fee)
        });

        const passphrase = await askPasspharaseFor(argv.account);

        const tx = createDelegateCCSTransaction(
            sdk,
            argv.delegatee,
            argv.quantity
        );
        const signed = await sdk.key.signTransaction(tx, {
            account: argv.account,
            passphrase,
            fee: argv.fee,
            seq: await sdk.rpc.chain.getSeq(argv.account)
        });
        console.log("Sending tx:", signed.hash().value);

        const newBlockNumber = await waitForTx(sdk, signed);
        console.log("Tx is contained in block #", newBlockNumber);

        await printSummary(sdk, newBlockNumber, argv.account, argv.delegatee);
    })
};

async function printSummary(
    sdk: SDK,
    blockNumber: number,
    account: PlatformAddress,
    delegatee: PlatformAddress,
    changes?: {
        ccsChanges: U64;
        cccChanges: U64;
    }
) {
    const { ccsChanges = new U64(0), cccChanges = new U64(0) } = changes || {};
    const summary = await summarize(sdk, blockNumber);

    console.group("Account", account.value);
    {
        const cccBalance = await sdk.rpc.chain.getBalance(account, blockNumber);
        const { balance, undelegated, delegationsTo } = summary.get(account);
        console.log("CCC Balance:", ...minusChangeArgs(cccBalance, cccChanges));
        console.log("CCS Balance:", balance.toLocaleString());
        console.log(
            "Undelegated CCS:",
            ...minusChangeArgs(undelegated, ccsChanges)
        );
        console.log(
            "Delegations (out):",
            ...plusChangeArgs(delegationsTo.sum, ccsChanges)
        );
    }
    console.groupEnd();

    console.group("Delegatee", delegatee.value);
    {
        const { balance, undelegated, delegationsFrom } = summary.get(
            delegatee
        );
        console.log("CCS Balance:", balance.toLocaleString());
        console.log("Undelegated CCS:", undelegated.toLocaleString());
        console.log(
            "Delegations (in):",
            ...plusChangeArgs(delegationsFrom.sum, ccsChanges)
        );
    }
    console.groupEnd();

    console.log(
        "Delegations between:",
        ...plusChangeArgs(summary.delegations(account, delegatee), ccsChanges)
    );
}
