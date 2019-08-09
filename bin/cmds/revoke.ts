import { SDK } from "codechain-sdk";
import { PlatformAddress, U64 } from "codechain-sdk/lib/core/classes";
import * as yargs from "yargs";

import { GlobalParams } from "..";
import { createRevokeTransaction } from "../../src";
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

interface RevokeParams extends GlobalParams {
    delegator: PlatformAddress;
    delegatee: PlatformAddress;
    quantity: U64;
    fee: number;
}

export const module: yargs.CommandModule<GlobalParams, RevokeParams> = {
    command: "revoke",
    describe: "Revoke delegation to an account",
    builder(args) {
        return args
            .option("delegator", {
                coerce: coerce("delegator", PlatformAddress.ensure),
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
                default: 0
            });
    },
    handler: asyncHandler(async argv => {
        const { sdk, blockNumber } = await prologue(argv);

        console.log("=== Confirm your action ===");
        console.log("Action:", "Revoke");
        console.log("Quantity:", argv.quantity.toString(10));
        await printSummary(sdk, blockNumber, argv.delegator, argv.delegatee, {
            ccsChanges: argv.quantity,
            cccChanges: U64.ensure(argv.fee)
        });

        const passphrase = await askPasspharaseFor(argv.delegator);

        const tx = createRevokeTransaction(sdk, argv.delegatee, argv.quantity);
        const signed = await sdk.key.signTransaction(tx, {
            account: argv.delegator,
            passphrase,
            fee: argv.fee,
            seq: await sdk.rpc.chain.getSeq(argv.delegator)
        });
        console.log("Sending tx:", signed.hash().value);

        const newBlockNumber = await waitForTx(sdk, signed);
        console.log("Tx is contained in block #", newBlockNumber);

        await printSummary(sdk, newBlockNumber, argv.delegator, argv.delegatee);
    })
};

async function printSummary(
    sdk: SDK,
    blockNumber: number,
    delegator: PlatformAddress,
    delegatee: PlatformAddress,
    changes?: {
        cccChanges: U64;
        ccsChanges: U64;
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
        const { balance, undelegated, delegationsTo } = summary.get(delegator);
        console.log("CCC Balance:", ...minusChangeArgs(cccBalance, cccChanges));
        console.log("CCS Balance:", balance.toLocaleString());
        console.log(
            "Undelegated CCS:",
            ...plusChangeArgs(undelegated, ccsChanges)
        );
        console.log(
            "Delegations (out):",
            ...minusChangeArgs(delegationsTo.sum, ccsChanges)
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
            ...minusChangeArgs(delegationsFrom.sum, ccsChanges)
        );
    }
    console.groupEnd();

    console.log(
        "Delegations between:",
        ...minusChangeArgs(
            summary.delegations(delegator, delegatee),
            ccsChanges
        )
    );
}
