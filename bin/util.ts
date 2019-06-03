import * as Table from "cli-table3";
import { HorizontalAlignment } from "cli-table3";
import { PlatformAddress, U64 } from "codechain-primitives";
import { SDK } from "codechain-sdk";
import { SignedTransaction } from "codechain-sdk/lib/core/classes";
import { KeyStoreType } from "codechain-sdk/lib/key";
import * as fs from "fs";
import * as yargs from "yargs";

const PromptPassword = require("prompt-password");

export async function newSDK(params: {
    server: string;
    keyStoreType?: KeyStoreType;
    options?: {
        transactionSigner?: string;
        fallbackServers?: string[];
    };
}) {
    const sdk = new SDK({ ...params, networkId: "cc" });
    const networkId = await sdk.rpc.chain.getNetworkId();
    return new SDK({ ...params, networkId });
}

export async function prologue(argv: {
    "rpc-server": string;
    "keys-path": string;
}) {
    const sdk = await newSDK({
        server: argv["rpc-server"],
        keyStoreType: {
            type: "local",
            path: argv["keys-path"]
        }
    });

    const networkId = await sdk.rpc.chain.getNetworkId();
    const { hash, number } = await sdk.rpc.chain.getBestBlockId();
    const block = (await sdk.rpc.chain.getBlock(number))!;
    const date = new Date(block.timestamp * 1000);
    console.log("RPC server:", argv["rpc-server"]);
    console.log("Network Id:", networkId);
    console.group("Current block:");
    console.log("Block number:", number);
    console.log("Block hash:", hash.toString());
    console.log("Block time:", date.toISOString());
    console.groupEnd();
    console.log();
    return {
        sdk,
        networkId,
        date,
        blockNumber: number,
        blockHash: hash
    };
}

export async function askPasspharaseFor(
    account: PlatformAddress
): Promise<string> {
    const prompt = new PromptPassword({
        type: "password",
        message: `To continue, enter passphrase for ${account.value}`,
        name: "password"
    });

    return new Promise((resolve, reject) =>
        prompt
            .run()
            .then(resolve)
            .catch(reject)
    );
}

export async function waitForTx(
    sdk: SDK,
    signed: SignedTransaction
): Promise<number> {
    const hash = await sdk.rpc.chain.sendSignedTransaction(signed);
    for (let retry = 0; ; retry++) {
        if (retry === 10) {
            throw new Error("Cannot fetch the transaction result");
        }

        if (await sdk.rpc.chain.containsTransaction(hash)) {
            const tx = (await sdk.rpc.chain.getTransaction(hash))!;
            return tx.blockNumber!;
        }
        const errorHint = await sdk.rpc.chain.getErrorHint(hash);
        if (errorHint != null) {
            console.error(signed.toJSON());
            throw new Error(errorHint);
        }
        await new Promise(resolve => setTimeout(resolve, 1000));
    }
}

export function createTable(
    head: string[],
    colAligns?: HorizontalAlignment[]
): any[][] {
    const table = new Table({
        chars: {
            top: "",
            "top-mid": "",
            "top-left": "",
            "top-right": "",
            bottom: "",
            "bottom-mid": "",
            "bottom-left": "",
            "bottom-right": "",
            left: "",
            "left-mid": "",
            mid: "",
            "mid-mid": "",
            right: "",
            "right-mid": "",
            middle: " "
        },
        style: { "padding-left": 0, "padding-right": 1 },
        head,
        colAligns: colAligns || []
    });
    return table as any;
}

export function percent(a: U64, b: U64): string {
    const digits = 1;
    return new U64(100).value
        .times(a.value)
        .div(b.value)
        .toFixed(digits);
}

export function plusChangeArgs(a: U64, b: U64): string[] {
    if (b.isEqualTo(0)) {
        return [a.toLocaleString()];
    } else {
        return [a.toLocaleString(), "=>", a.plus(b).toLocaleString()];
    }
}

export function minusChangeArgs(a: U64, b: U64): string[] {
    if (b.isEqualTo(0)) {
        return [a.toLocaleString()];
    } else {
        if (a.isGreaterThanOrEqualTo(b)) {
            return [a.toLocaleString(), "=>", a.minus(b).toLocaleString()];
        } else {
            return [
                a.toLocaleString(),
                "=>",
                "-" + b.minus(a).toLocaleString()
            ];
        }
    }
}

export function humanizeTimstamp(
    ts: number,
    options?: { current?: number; precision?: number; limit?: number }
): string | null {
    const {
        current = Date.now() / 1000,
        precision = 2,
        limit = 60 * 60 * 24 * 7 * 4
    } = options || {};

    let offset = Math.abs(current - ts);
    if (offset < 1) {
        return `now`;
    } else if (offset >= limit) {
        return null;
    }

    const table: [number, string, string][] = [
        [60 * 60 * 24 * 7, "week", "weeks"],
        [60 * 60 * 24, "day", "days"],
        [60 * 60, "hour", "hours"],
        [60, "minute", "minutes"],
        [1, "second", "seconds"]
    ];
    const result = [];
    let firstIndex = -1;
    for (let i = 0; i < table.length; i++) {
        if (firstIndex !== -1 && i - firstIndex === precision) {
            break;
        }
        const [interval, singular, plural] = table[i];
        if (offset >= interval) {
            const units = Math.floor(offset / interval);
            offset %= interval;
            if (units === 1) {
                result.push(`${units} ${singular}`);
            } else {
                result.push(`${units} ${plural}`);
            }
            if (firstIndex === -1) {
                firstIndex = i;
            }
        }
    }

    let postfix: string;
    if (current > ts) {
        postfix = "ago";
    } else {
        postfix = "after";
    }
    return `${result.join(" ")} ${postfix}`;
}

export function formatTimestamp(
    ts: number,
    options: { current?: number; precision?: number; limit?: number } = {}
): string {
    const date = new Date(ts * 1000).toISOString();
    const humanized = humanizeTimstamp(ts, options);
    if (humanized) {
        return `${date} (${humanized})`;
    } else {
        return date;
    }
}

export function asyncHandler<T>(
    handler: (argv: T) => Promise<void>
): (argv: T) => Promise<void> {
    return async argv => {
        yargs.showHelpOnFail(false);
        try {
            await handler(argv);
        } catch (e) {
            console.error("Error:", e.message);
            if (process.env.STACK_TRACE_CCSTAKE) {
                fs.writeFileSync("./ccstake-error.log", e.stack);
            }
            yargs.exit(-1, e);
        }
    };
}

export function coerce<T>(
    name: string,
    func: (arg: any) => T
): (arg: any) => T {
    return (arg: any) => {
        try {
            return func(arg);
        } catch (e) {
            throw new Error(`Invalid option "${name}":\n${e.message}`);
        }
    };
}
