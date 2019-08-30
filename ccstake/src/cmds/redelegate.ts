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
    delegator: PlatformAddress;
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
            .option("delegator", {
                coerce: coerce("delegator", PlatformAddress.ensure),
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
            argv.delegator,
            argv["previous-delegatee"],
            argv["next-delegatee"],
            {
                ccsChanges: argv.quantity,
                cccChanges: U64.ensure(argv.fee)
            }
        );

        const passphrase = await askPasspharaseFor(argv.delegator);

        const tx = createRedelegateTransaction(
            sdk,
            argv["previous-delegatee"],
            argv["next-delegatee"],
            argv.quantity
        );
        const signed = await sdk.key.signTransaction(tx, {
            account: argv.delegator,
            passphrase,
            fee: argv.fee,
            seq: await sdk.rpc.chain.getSeq(argv.delegator)
        });
        console.log("Sending tx:", signed.hash().value);

        const newBlockNumber = await waitForTx(sdk, signed);
        console.log("Tx is contained in block #", newBlockNumber);

        await printSummary(
            sdk,
            newBlockNumber,
            argv.delegator,
            argv["previous-delegatee"],
            argv["next-delegatee"]
        );
    })
};

async function printSummary(
    sdk: SDK,
    blockNumber: number,
    delegator: PlatformAddress,
    previousDelegatee: PlatformAddress,
    nextDelegatee: PlatformAddress,
    changes?: {
        ccsChanges: U64;
        cccChanges: U64;
    }
) {
    const { ccsChanges = new U64(0), cccChanges = new U64(0) } = changes || {};
    const summary = await summarize(sdk, blockNumber);

    console.group("Delegator", delegator.value);
    {
        const cccBalance = await sdk.rpc.chain.getBalance(
            delegator,
            blockNumber
        );
        console.log("CCC Balance:", ...minusChangeArgs(cccBalance, cccChanges));
        const { balance, undelegated, delegationsTo } = summary.get(delegator);
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
        `Delegations between ${delegator.value} ${previousDelegatee.value} (previous delegator):`,
        ...minusChangeArgs(
            summary.delegations(delegator, previousDelegatee),
            ccsChanges
        )
    );

    console.log(
        `Delegations between ${delegator.value} ${nextDelegatee.value} (next delegator):`,
        ...plusChangeArgs(
            summary.delegations(delegator, nextDelegatee),
            ccsChanges
        )
    );
}
