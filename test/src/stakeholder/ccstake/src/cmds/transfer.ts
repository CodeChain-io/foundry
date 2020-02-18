import { SDK } from "codechain-sdk";
import { PlatformAddress, U64 } from "codechain-sdk/lib/core/classes";
import {
    createTransferCCSTransaction,
    getDelegations,
    getUndelegatedCCS
} from "codechain-stakeholder-sdk";
import * as yargs from "yargs";

import { GlobalParams } from "..";
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
    account: PlatformAddress;
    recipient: PlatformAddress;
    quantity: U64;
    fee: number;
}

export const module: yargs.CommandModule<GlobalParams, TransferParams> = {
    command: "transfer",
    describe: "Transfer CCS to an account",
    builder(args) {
        return args
            .option("account", {
                coerce: coerce("account", PlatformAddress.ensure),
                demand: true
            })
            .option("recipient", {
                coerce: coerce("recipient", PlatformAddress.ensure),
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
        console.log("Action:", "Transfer");
        console.log("Quantity:", argv.quantity.toLocaleString());
        await printSummary(sdk, blockNumber, argv.account, argv.recipient, {
            ccsChanges: argv.quantity,
            cccChanges: U64.ensure(argv.fee)
        });

        const passphrase = await askPasspharaseFor(argv.account);

        const tx = createTransferCCSTransaction(
            sdk,
            argv.recipient,
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

        console.log("Balances after action");
        await printSummary(sdk, newBlockNumber, argv.account, argv.recipient);
    })
};

async function printSummary(
    sdk: SDK,
    blockNumber: number,
    account: PlatformAddress,
    recipient: PlatformAddress,
    changes?: {
        ccsChanges: U64;
        cccChanges: U64;
    }
) {
    const { ccsChanges = new U64(0), cccChanges = new U64(0) } = changes || {};

    console.group("Account", account.value);
    {
        const cccBalance = await sdk.rpc.chain.getBalance(account, blockNumber);
        const undelegated = await getUndelegatedCCS(sdk, account, blockNumber);
        const delegations = await getDelegations(sdk, account, blockNumber);
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

    console.group("Recipient", recipient.value);
    {
        const undelegated = await getUndelegatedCCS(
            sdk,
            recipient,
            blockNumber
        );
        const delegations = await getDelegations(sdk, recipient, blockNumber);
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
