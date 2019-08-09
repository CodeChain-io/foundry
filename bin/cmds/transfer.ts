import { SDK } from "codechain-sdk";
import { PlatformAddress, U64 } from "codechain-sdk/lib/core/classes";
import * as yargs from "yargs";

import { GlobalParams } from "..";
import {
    createTransferCCSTransaction,
    getDelegations,
    getUndelegatedCCS
} from "../../src";
import { sumU64 } from "../summerizer";
import {
    askPasspharaseFor,
    asyncHandler,
    coerce,
    minusChangeArgs,
    plusChangeArgs,
    prologue,
    waitForTx
} from "../util";

interface TransferParams extends GlobalParams {
    from: PlatformAddress;
    to: PlatformAddress;
    quantity: U64;
    fee: number;
}

export const module: yargs.CommandModule<GlobalParams, TransferParams> = {
    command: "transfer",
    describe: "Transfer CCS to an account",
    builder(args) {
        return args
            .option("from", {
                coerce: coerce("from", PlatformAddress.ensure),
                demand: true
            })
            .option("to", {
                coerce: coerce("to", PlatformAddress.ensure),
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
        console.log("Action:", "Transfer");
        console.log("Quantity:", argv.quantity.toLocaleString());
        await printSummary(sdk, blockNumber, argv.from, argv.to, {
            ccsChanges: argv.quantity,
            cccChanges: U64.ensure(argv.fee)
        });

        const passphrase = await askPasspharaseFor(argv.from);

        const tx = createTransferCCSTransaction(sdk, argv.to, argv.quantity);
        const signed = await sdk.key.signTransaction(tx, {
            account: argv.from,
            passphrase,
            fee: argv.fee,
            seq: await sdk.rpc.chain.getSeq(argv.from)
        });
        console.log("Sending tx:", signed.hash().value);

        const newBlockNumber = await waitForTx(sdk, signed);
        console.log("Tx is contained in block #", newBlockNumber);

        console.log("Balances after action");
        await printSummary(sdk, newBlockNumber, argv.from, argv.to);
    })
};

async function printSummary(
    sdk: SDK,
    blockNumber: number,
    sender: PlatformAddress,
    receiver: PlatformAddress,
    changes?: {
        ccsChanges: U64;
        cccChanges: U64;
    }
) {
    const { ccsChanges = new U64(0), cccChanges = new U64(0) } = changes || {};

    console.group("Sender", sender.value);
    {
        const cccBalance = await sdk.rpc.chain.getBalance(sender, blockNumber);
        const undelegated = await getUndelegatedCCS(sdk, sender, blockNumber);
        const delegations = await getDelegations(sdk, sender, blockNumber);
        const balance = undelegated.plus(
            sumU64(delegations.map(x => x.quantity))
        );
        console.log("CCC Balance:", ...minusChangeArgs(cccBalance, cccChanges));
        console.log("CCS Balance:", ...minusChangeArgs(balance, ccsChanges));
        console.log(
            "Undelegated CCS:",
            ...minusChangeArgs(undelegated, ccsChanges)
        );
    }
    console.groupEnd();

    console.group("Receiver", receiver.value);
    {
        const undelegated = await getUndelegatedCCS(sdk, receiver, blockNumber);
        const delegations = await getDelegations(sdk, receiver, blockNumber);
        const balance = undelegated.plus(
            sumU64(delegations.map(x => x.quantity))
        );
        console.log("CCS Balance:", ...plusChangeArgs(balance, ccsChanges));
        console.log(
            "Undelegated CCS:",
            ...plusChangeArgs(undelegated, ccsChanges)
        );
    }
    console.groupEnd();
}
