import { PlatformAddress, U64 } from "codechain-primitives/lib";
import { SDK } from "codechain-sdk";
import * as yargs from "yargs";

import { GlobalParams } from "..";
import { createRequestRevokeTransaction } from "../../src";
import { summarize } from "../summerizer";
import {
    askPasspharaseFor,
    asyncHandler,
    coerce,
    formatTimestamp,
    plusChangeArgs,
    prologue,
    waitForTx
} from "../util";

interface RequestRevokeParams extends GlobalParams {
    delegator: PlatformAddress;
    delegatee: PlatformAddress;
    quantity: U64;
}

export const module: yargs.CommandModule<GlobalParams, RequestRevokeParams> = {
    command: "request-revoke",
    describe: "Request revoke delegation to an account",
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
            });
    },
    handler: asyncHandler(async argv => {
        const { sdk, blockNumber } = await prologue(argv);

        console.log("=== Confirm your action ===");
        console.log("Action:", "RequestRevoke");
        console.log("Quantity:", argv.quantity.toString(10));
        await printSummary(
            sdk,
            blockNumber,
            argv.delegator,
            argv.delegatee,
            argv.quantity
        );

        const passphrase = await askPasspharaseFor(argv.delegator);

        const tx = createRequestRevokeTransaction(
            sdk,
            argv.delegatee,
            argv.quantity
        );
        const signed = await sdk.key.signTransaction(tx, {
            account: argv.delegator,
            passphrase,
            fee: 10,
            seq: await sdk.rpc.chain.getSeq(argv.delegator)
        });
        console.log("Sending tx:", signed.hash().value);

        const newBlockNumber = await waitForTx(sdk, signed);
        console.log("Tx is contained in block #", newBlockNumber);

        const summary = await printSummary(
            sdk,
            newBlockNumber,
            argv.delegator,
            argv.delegatee
        );

        const revocations = summary.revocations(argv.delegator, argv.delegatee)
            .values;
        const block = (await sdk.rpc.chain.getBlock(blockNumber))!;
        const lastEndTime = revocations[revocations.length - 1].endTime;
        console.log(
            "Delegation will be revoked at",
            formatTimestamp(lastEndTime, {
                current: block.timestamp
            })
        );
    })
};

async function printSummary(
    sdk: SDK,
    blockNumber: number,
    delegator: PlatformAddress,
    delegatee: PlatformAddress,
    changes?: U64
) {
    const summary = await summarize(sdk, blockNumber);

    console.group("Delegator", delegator.value);
    {
        const {
            balance,
            undelegated,
            delegationsTo,
            revocationsTo
        } = summary.get(delegator);
        console.log("CCS Balance:", balance.toLocaleString());
        console.log("Undelegated CCS:", undelegated.toLocaleString());
        console.log("Delegations (out):", delegationsTo.sum.toLocaleString());
        console.log(
            "Pending Revocations (out):",
            ...plusChangeArgs(revocationsTo.sum, changes)
        );
    }
    console.groupEnd();

    console.group("Delegatee", delegatee.value);
    {
        const {
            balance,
            undelegated,
            delegationsFrom,
            revocationsFrom
        } = summary.get(delegatee);
        console.log("CCS Balance:", balance.toLocaleString());
        console.log("Undelegated CCS:", undelegated.toLocaleString());
        console.log("Delegations (in):", delegationsFrom.sum.toLocaleString());
        console.log(
            "Pending Revocations (in):",
            ...plusChangeArgs(revocationsFrom.sum, changes)
        );
    }
    console.groupEnd();

    console.log(
        "Delegations between:",
        summary.delegations(delegator, delegatee).toLocaleString()
    );
    console.log(
        "Pending revocations between:",
        ...plusChangeArgs(
            summary.revocations(delegator, delegatee).sum,
            changes
        )
    );

    return summary;
}
