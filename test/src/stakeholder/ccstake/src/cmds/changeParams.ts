import { SDK } from "codechain-sdk";
import { PlatformAddress, U64 } from "codechain-sdk/lib/core/classes";
import * as yargs from "yargs";
const RLP = require("rlp");

import { GlobalParams } from "..";
import {
    askPasspharaseFor,
    asyncHandler,
    coerce,
    minusChangeArgs,
    prologue,
    waitForTx
} from "../util";

interface ChangeParams extends GlobalParams {
    transaction: Buffer;
    account: PlatformAddress;
    fee: number;
}

export const module: yargs.CommandModule<GlobalParams, ChangeParams> = {
    command: "change-params",
    describe: "Change CodeChain network parameter",
    builder(args) {
        return args
            .option("transaction", {
                coerce: coerce("transaction", input => {
                    const encodedTransaction = Buffer.from(input, "hex");
                    ensureBasicCheck(encodedTransaction);
                    return encodedTransaction;
                }),
                demand: true
            })
            .option("account", {
                coerce: coerce("account", PlatformAddress.ensure),
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
        console.log("Action:", "ChangeParams");
        console.log("Encoded transaction:", argv.transaction.toString("hex"));
        await printSummary(sdk, blockNumber, argv.account, argv.fee);

        const passphrase = await askPasspharaseFor(argv.account);

        const tx = sdk.core.createCustomTransaction({
            handlerId: 2,
            bytes: argv.transaction
        });
        const signed = await sdk.key.signTransaction(tx, {
            account: argv.account,
            passphrase,
            fee: argv.fee,
            seq: await sdk.rpc.chain.getSeq(argv.account)
        });
        console.log("Sending tx:", signed.hash().value);

        const newBlockNumber = await waitForTx(sdk, signed);
        console.log("Tx is contained in block #", newBlockNumber);

        await printSummary(sdk, newBlockNumber, argv.account);
    })
};

function ensureBasicCheck(encodedTransaction: Buffer) {
    let decoded;
    try {
        decoded = RLP.decode(encodedTransaction);
    } catch (e) {
        console.error(e);
        throw new Error("Error in RLP format");
    }
    if (
        !Array.isArray(decoded) ||
        decoded.length <= 3 ||
        !decodeU64(decoded[0]).eq(0xff)
    ) {
        throw new Error("Invalid ChangeParams transaction");
    }
}

async function printSummary(
    sdk: SDK,
    blockNumber: number,
    account: PlatformAddress,
    cccChange: number = 0
) {
    console.group("Account", account.toString());
    {
        const cccBalance = await sdk.rpc.chain.getBalance(account, blockNumber);
        console.log(
            "CCC Balance:",
            ...minusChangeArgs(cccBalance, new U64(cccChange))
        );
    }
    console.groupEnd();
}

function decodeU64(buffer: Buffer): U64 {
    if (buffer.length === 0) {
        return new U64(0);
    }
    return U64.ensure("0x" + buffer.toString("hex"));
}
