import { SDK } from "codechain-sdk";
import { PlatformAddress, U64 } from "codechain-sdk/lib/core/classes";
import { createRedelegateTransaction } from "codechain-stakeholder-sdk";
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

interface RedelegateParams extends GlobalParams {
    account: PlatformAddress;
    "previous-delegatee": PlatformAddress;
    "next-delegatee": PlatformAddress;
    quantity: U64;
    fee: number;
}

export const module: yargs.CommandModule<GlobalParams, RedelegateParams> = {
    command: "redelegate",
    describe: "Move a delegation to another account",
    builder(args) {
        return args
            .option("account", {
                coerce: coerce("account", PlatformAddress.ensure),
                demand: true
            })
            .option("previous-delegatee", {
                coerce: coerce("previous-delegatee", PlatformAddress.ensure),
                demand: true
            })
            .option("next-delegatee", {
                coerce: coerce("next-delegatee", PlatformAddress.ensure),
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
        console.log("Action:", "Redelegate");
        console.log("Quantity:", argv.quantity.toLocaleString());
        await printSummary(
            sdk,
            blockNumber,
            argv.account,
            argv["previous-delegatee"],
            argv["next-delegatee"],
            {
                ccsChanges: argv.quantity,
                cccChanges: U64.ensure(argv.fee)
            }
        );

        const passphrase = await askPasspharaseFor(argv.account);

        const tx = createRedelegateTransaction(
            sdk,
            argv["previous-delegatee"],
            argv["next-delegatee"],
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

        await printSummary(
            sdk,
            newBlockNumber,
            argv.account,
            argv["previous-delegatee"],
            argv["next-delegatee"]
        );
    })
};

async function printSummary(
    sdk: SDK,
    blockNumber: number,
    account: PlatformAddress,
    previousDelegatee: PlatformAddress,
    nextDelegatee: PlatformAddress,
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
        console.log("CCC Balance:", ...minusChangeArgs(cccBalance, cccChanges));
        const { balance, undelegated, delegationsTo } = summary.get(account);
        console.log("CCS Balance:", balance.toLocaleString());
        console.log("Undelegated CCS:", undelegated.toLocaleString());
        console.log("Delegations (out):", delegationsTo.sum.toLocaleString());
    }
    console.groupEnd();

    console.group("Previous delegatee", previousDelegatee.value);
    {
        const { balance, undelegated, delegationsFrom } = summary.get(
            previousDelegatee
        );
        console.log("CCS Balance:", balance.toLocaleString());
        console.log("Undelegated CCS:", undelegated.toLocaleString());
        console.log(
            "Delegations (in):",
            ...minusChangeArgs(delegationsFrom.sum, ccsChanges)
        );
    }
    console.groupEnd();

    console.group("Next delegatee", nextDelegatee.value);
    {
        const { balance, undelegated, delegationsFrom } = summary.get(
            nextDelegatee
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
        `Delegations between ${account.value} ${previousDelegatee.value} (previous delegatee):`,
        ...minusChangeArgs(
            summary.delegations(account, previousDelegatee),
            ccsChanges
        )
    );

    console.log(
        `Delegations between ${account.value} ${nextDelegatee.value} (next delegatee):`,
        ...plusChangeArgs(
            summary.delegations(account, nextDelegatee),
            ccsChanges
        )
    );
}
