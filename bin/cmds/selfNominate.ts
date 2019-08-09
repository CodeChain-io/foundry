import { SDK } from "codechain-sdk";
import { PlatformAddress, U64 } from "codechain-sdk/lib/core/classes";
import * as yargs from "yargs";

import { GlobalParams } from "..";
import { createSelfNominateTransaction } from "../../src";
import {
    askPasspharaseFor,
    asyncHandler,
    coerce,
    minusChangeArgs,
    prologue,
    waitForTx
} from "../util";

type MetadataType = "hex" | "text";
interface SelfNominateParams extends GlobalParams {
    account: PlatformAddress;
    deposit: U64;
    metadata: string;
    "metadata-type": MetadataType;
    fee: number;
}

export const module: yargs.CommandModule<GlobalParams, SelfNominateParams> = {
    command: "self-nominate",
    describe: "Self nominate as a candidate",
    builder(args) {
        return args
            .option("account", {
                describe: "An account to self-nominate",
                coerce: coerce("account", PlatformAddress.ensure),
                demand: true
            })
            .option("deposit", {
                describe: "Deposit in CCC to self-nominate",
                coerce: coerce("deposit", U64.ensure),
                demand: true
            })
            .option("metadata", {
                describe: "A hex-string or a plain text metadata",
                coerce: coerce("metadata", x => {
                    if (x === null || x === undefined) {
                        return "";
                    } else {
                        return x;
                    }
                }),
                demand: true
            })
            .option("metadata-type", {
                choices: ["hex", "text"],
                default: "text" as MetadataType
            })
            .option("fee", {
                number: true,
                demand: true
            });
    },
    handler: asyncHandler(async argv => {
        const metadata = normalizeMetadata(
            argv.metadata,
            argv["metadata-type"]
        );
        const { sdk, blockNumber } = await prologue(argv);

        console.log("=== Confirm your action ===");
        console.log("Action:", "SelfNominate");
        console.log("Candidate:", argv.account.value);
        console.log("Deposit:", argv.deposit.toLocaleString());
        console.group("Metadata:");
        metadata.print();
        console.groupEnd();

        await printSummary(
            sdk,
            blockNumber,
            argv.account,
            argv.deposit.plus(argv.fee)
        );

        const passphrase = await askPasspharaseFor(argv.account);

        const tx = createSelfNominateTransaction(
            sdk,
            argv.deposit,
            metadata.buffer
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

        await printSummary(sdk, newBlockNumber, argv.account);
    })
};

async function printSummary(
    sdk: SDK,
    blockNumber: number,
    account: PlatformAddress,
    cccChanges?: U64
) {
    console.group("Account", account.value);
    {
        const ccc = await sdk.rpc.chain.getBalance(account, blockNumber);
        console.log(
            "CCC Balance:",
            ...minusChangeArgs(ccc, cccChanges || new U64(0))
        );
    }
    console.groupEnd();
}

interface NormalizedMetadata {
    buffer: Buffer;
    print(): void;
}

function normalizeMetadata(
    metadata: string,
    designatedType: MetadataType
): NormalizedMetadata {
    if (metadata.length === 0) {
        return {
            buffer: Buffer.from([]),
            print() {
                console.log("Payload:", "null");
                console.log("Length:", 0, "bytes");
            }
        };
    }
    if (designatedType === "text") {
        const buffer = Buffer.from(metadata, "utf8");
        return {
            buffer,
            print() {
                console.log("Payload (text):", metadata);
                console.log("Length:", buffer.length, "bytes");
            }
        };
    } else if (designatedType === "hex") {
        metadata = metadata.trim();
        if (!/^(0x)?[0-9a-fA-F]*$/.test(metadata)) {
            throw new Error("A metadata contains hex characters");
        }
        if (metadata.length % 2 !== 0) {
            throw new Error("A length of a metadata is not even");
        }
        if (metadata.startsWith("0x")) {
            metadata = metadata.substr(2);
        }
        const buffer = Buffer.from(metadata, "hex");
        return {
            buffer,
            print() {
                console.log("Payload (hex):", "0x" + metadata);
                console.log("Length:", buffer.length, "bytes");
            }
        };
    } else {
        throw new Error("Invalid metadataType");
    }
}
