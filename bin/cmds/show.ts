import { PlatformAddress } from "codechain-primitives";
import { SDK } from "codechain-sdk";
import * as yargs from "yargs";

import { GlobalParams } from "../index";
import { summarize } from "../summerizer";
import { asyncHandler, coerce, createTable, percent, prologue } from "../util";

interface ShowParams extends GlobalParams {
    account?: PlatformAddress;
}

export const module: yargs.CommandModule<GlobalParams, ShowParams> = {
    command: "show [account]",
    describe: "Show staking status of an account",
    builder(args) {
        return args.positional("account", {
            describe: "An account to show",
            type: "string",
            coerce: coerce("account", (account: string | undefined) => {
                if (account !== undefined) {
                    return PlatformAddress.ensure(account);
                } else {
                    return undefined;
                }
            })
        });
    },
    handler: asyncHandler(async argv => {
        const { sdk, blockNumber } = await prologue(argv);

        if (argv.account) {
            await show(sdk, argv.account, blockNumber);
        } else {
            await overview(sdk, blockNumber);
        }
    })
};

async function show(sdk: SDK, account: PlatformAddress, blockNumber: number) {
    console.log(`Staking summary of ${account.value}`);
    console.log();
    const summaryAll = await summarize(sdk, blockNumber);
    const summary = summaryAll.get(account);

    /* balance */
    const balance = summary.balance;
    const totalCCS = summaryAll.totalCCS;
    const share = percent(balance, totalCCS);
    console.log(
        `CCS balance: ${balance.toLocaleString()} of ${totalCCS.toLocaleString()} (about ${share.toLocaleString()}%)`
    );

    console.log(`Undelegated CCS: ${summary.undelegated.toLocaleString()}`);

    console.group(
        `Delegations to: Total ${summary.delegationsTo.sum.toLocaleString()} CCS`
    );
    if (summary.delegationsTo.values.length > 0) {
        const table = createTable(["Delegatee", "Quantity"], ["left", "right"]);
        for (const { delegatee, quantity } of summary.delegationsTo.values) {
            table.push([delegatee.value, quantity.toLocaleString()]);
        }
        console.log(table.toString());
    }
    console.groupEnd();

    console.group(
        `Delegations from: Total ${summary.delegationsFrom.sum.toLocaleString()} CCS`
    );
    if (summary.delegationsFrom.values.length > 0) {
        const table = createTable(["Delegator", "Quantity"], ["left", "right"]);
        for (const { delegator, quantity } of summary.delegationsFrom.values) {
            table.push([delegator.value, quantity.toLocaleString()]);
        }
        console.log(table.toString());
    }
    console.groupEnd();
}

async function overview(sdk: SDK, blockNumber: number) {
    console.log(`Staking overview`);
    console.log();

    const summary = await summarize(sdk, blockNumber);

    console.log("Total CCS:", summary.totalCCS.toLocaleString());
    const table = createTable(
        [
            "Amount",
            "%",
            "CCS",
            "Undelegated",
            "Delegations\n(Out)",
            "Delegations\n(In)"
        ],
        ["left", "right", "right", "right", "right", "right", "right", "right"]
    );
    for (const account of summary.ccsHolders) {
        const { undelegated, delegationsTo, delegationsFrom } = summary.get(
            account
        );
        const balance = undelegated.plus(delegationsTo.sum);
        const share = percent(balance, summary.totalCCS);
        table.push([
            account.value,
            share.toString() + "%",
            balance.toLocaleString(),
            undelegated.toLocaleString(),
            delegationsTo.sum.toLocaleString(),
            delegationsFrom.sum.toLocaleString()
        ]);
    }
    console.log(table.toString());
}
