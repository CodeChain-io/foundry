import { SDK } from "codechain-sdk";
import {
    PlatformAddress,
    SignedTransaction,
    U64
} from "codechain-sdk/lib/core/classes";
import * as stake from "codechain-stakeholder-sdk";
import * as fs from "fs";
import * as util from "util";
import * as yargs from "yargs";

import prompts = require("prompts");
import { GlobalParams } from "..";
import { summarize, Summary } from "../summerizer";
import { asyncHandler, createTable, prologue } from "../util";

interface BatchDelegateParams extends GlobalParams {
    "distribution-file": string;
    "password-path": string;
    "dry-run": boolean;
}

export const module: yargs.CommandModule<GlobalParams, BatchDelegateParams> = {
    command: "batch-delegate <distribution-file>",
    describe: "Batch manage delegations through distribution file",
    builder(args) {
        return args
            .positional("distribution-file", {
                desc: "File to describe the distribution of delegations",
                type: "string",
                normalize: true
            })
            .demandOption("distribution-file")
            .option("password-path", {
                desc: "Path to password file to unlock accounts",
                string: true,
                normalize: true,
                demand: true
            })
            .option("dry-run", {
                desc: "Do not execute transactions",
                boolean: true,
                default: false
            });
    },
    handler: asyncHandler(async argv => {
        const input = await BatchDelegation.fromFile(argv["distribution-file"]);
        const passwords = await passwordsFromFile(argv["password-path"]);
        const dryRun = argv["dry-run"];

        const { sdk, blockNumber } = await prologue(argv);
        const summary = await summarize(sdk, blockNumber);

        preCheck(input, summary);
        const planned = await plan(input, summary);
        printPlan(planned);
        await confirm();
        await check(sdk, planned);
        await execute(sdk, passwords, input.stakeholders, planned, dryRun);
    })
};

function preCheck(input: BatchDelegation, summary: Summary) {
    const totalCCSToDelegate = input.distributions
        .map(x => x.quantity)
        .reduce((a: U64, b: U64) => a.plus(b), new U64(0));

    let availableCCS = new U64(0);
    for (const stakeholder of input.stakeholders) {
        availableCCS = availableCCS.plus(summary.get(stakeholder).undelegated);
        for (const delegatee of input.validators) {
            availableCCS = availableCCS.plus(
                summary.delegations(stakeholder, delegatee)
            );
        }
    }

    if (availableCCS.lt(totalCCSToDelegate)) {
        throw new Error(
            `stakeholders' available CCS (${availableCCS.toLocaleString()}) are less than the sum of distributions (${totalCCSToDelegate.toLocaleString()})`
        );
    }
}

type Tx = { delegator: string; quantity: U64; fee: number } & (
    | { type: "delegate"; delegatee: string }
    | { type: "redelegate"; prevDelegatee: string; nextDelegatee: string }
    | { type: "revoke"; delegatee: string });

async function plan(input: BatchDelegation, summary: Summary): Promise<Tx[]> {
    function setOrRemoveIfZero(map: Map<string, U64>, key: string, value: U64) {
        if (value.eq(0)) {
            map.delete(key);
        } else {
            map.set(key, value);
        }
    }
    // tracer is like a mutable summary.
    const tracer = {
        transactions: [] as Tx[],
        undelegateds: new Map<string, U64>(),
        undelegated(delegator: PlatformAddress) {
            const key = delegator.toString();
            const self = this;
            return {
                get value() {
                    return self.undelegateds.get(key) || new U64(0);
                },
                set value(quantity: U64) {
                    setOrRemoveIfZero(self.undelegateds, key, quantity);
                }
            };
        },
        delegations: new Map<string, U64>(),
        delegation(delegator: PlatformAddress, delegatee: PlatformAddress) {
            const key = `${delegator.toString()}${delegatee.toString()}`;
            const self = this;
            return {
                get value() {
                    return self.delegations.get(key) || new U64(0);
                },
                set value(quantity: U64) {
                    setOrRemoveIfZero(self.delegations, key, quantity);
                },
                plus(quantity: U64) {
                    this.value = this.value.plus(quantity);
                },
                minus(quantity: U64) {
                    this.value = this.value.minus(quantity);
                }
            };
        },
        getNeedRevokes() {
            const result = [];
            for (const { validator, quantity } of input.distributions) {
                const delegations = input.stakeholders
                    .map(x => this.delegation(x, validator).value)
                    .reduce(U64.plus, new U64(0));
                if (delegations.gt(quantity)) {
                    result.push({
                        validator,
                        toRevoke: delegations.minus(quantity)
                    });
                }
            }
            return result;
        },
        getNeedDelegations() {
            const result = [];
            for (const { validator, quantity } of input.distributions) {
                const delegations = input.stakeholders
                    .map(x => this.delegation(x, validator).value)
                    .reduce(U64.plus, new U64(0));
                if (quantity.gt(delegations)) {
                    result.push({
                        validator,
                        toDelegate: quantity.minus(delegations)
                    });
                }
            }
            return result;
        }
    };
    // initialize tracer from summary
    for (const stakeholder of input.stakeholders) {
        tracer.undelegated(stakeholder).value = summary.get(
            stakeholder
        ).undelegated;
        for (const validator of input.validators) {
            const delegation = summary.delegations(stakeholder, validator);
            tracer.delegation(stakeholder, validator).value = delegation;
        }
    }

    function redelegate(
        delegator: PlatformAddress,
        prevDelegatee: PlatformAddress,
        nextDelegatee: PlatformAddress,
        quantity: U64
    ) {
        if (quantity.eq(0)) {
            return;
        }
        tracer.delegation(delegator, prevDelegatee).minus(quantity);
        tracer.delegation(delegator, nextDelegatee).plus(quantity);
        tracer.transactions.push({
            type: "redelegate",
            delegator: delegator.toString(),
            prevDelegatee: prevDelegatee.toString(),
            nextDelegatee: nextDelegatee.toString(),
            quantity,
            fee: input.fee
        });
    }
    function revoke(
        delegator: PlatformAddress,
        delegatee: PlatformAddress,
        quantity: U64
    ) {
        if (quantity.eq(0)) {
            return;
        }
        const undelegated = tracer.undelegated(delegator);
        undelegated.value = undelegated.value.plus(quantity);
        tracer.delegation(delegator, delegatee).minus(quantity);
        tracer.transactions.push({
            type: "revoke",
            delegator: delegator.toString(),
            delegatee: delegatee.toString(),
            quantity,
            fee: input.fee
        });
    }
    function delegate(
        delegator: PlatformAddress,
        delegatee: PlatformAddress,
        quantity: U64
    ) {
        if (quantity.eq(0)) {
            return;
        }
        const undelegated = tracer.undelegated(delegator);
        undelegated.value = undelegated.value.minus(quantity);
        tracer.delegation(delegator, delegatee).plus(quantity);
        tracer.transactions.push({
            type: "delegate",
            delegator: delegator.toString(),
            delegatee: delegatee.toString(),
            quantity,
            fee: input.fee
        });
    }

    function cap(upperLimit: U64, baseline: U64, want: U64) {
        if (baseline.plus(want).lt(upperLimit)) {
            return want;
        } else {
            return upperLimit.minus(baseline);
        }
    }

    // Plan Redelegation
    const min = (lhs: U64, rhs: U64) => (lhs.gt(rhs) ? lhs : rhs);
    for (const needRevoke of tracer.getNeedRevokes()) {
        // Greedy: First come, first redelegate.
        const { validator: prev, toRevoke } = needRevoke;
        let accumulated = new U64(0);
        revokeAll: for (const needDelegation of tracer.getNeedDelegations()) {
            const { validator: next, toDelegate } = needDelegation;
            const toRedelegate = min(toRevoke, toDelegate);
            for (const stakeholder of input.stakeholders) {
                const delegation = tracer.delegation(stakeholder, prev).value;
                const quantity = cap(toRedelegate, accumulated, delegation);
                accumulated = accumulated.plus(quantity);
                redelegate(stakeholder, prev, next, quantity);
                if (accumulated.isEqualTo(toRevoke)) {
                    break revokeAll; // to get next needRevoke;
                } else if (accumulated.isEqualTo(toDelegate)) {
                    break; // to get next needDelegation;
                }
            }
        }
    }

    // Plan Revoke
    for (const { validator, toRevoke } of tracer.getNeedRevokes()) {
        let accumulated = new U64(0);
        // Greedy: First come, first revoke.
        for (const stakeholder of input.stakeholders) {
            const delegation = tracer.delegation(stakeholder, validator).value;
            const quantity = cap(toRevoke, accumulated, delegation);
            accumulated = accumulated.plus(quantity);
            revoke(stakeholder, validator, quantity);
            if (accumulated.isEqualTo(toRevoke)) {
                break;
            }
        }
    }

    // Plan Delegate
    for (const { validator, toDelegate } of tracer.getNeedDelegations()) {
        let accumulated = new U64(0);
        // Greedy: First come, first delegate.
        for (const stakeholder of input.stakeholders) {
            const undelegated = tracer.undelegated(stakeholder).value;
            const quantity = cap(toDelegate, accumulated, undelegated);
            accumulated = accumulated.plus(quantity);
            delegate(stakeholder, validator, quantity);
            if (accumulated.isEqualTo(toDelegate)) {
                break;
            }
        }
    }

    return tracer.transactions;
}

async function printPlan(planned: Tx[]) {
    const table = createTable([
        "Id",
        "Action",
        "Quantity",
        "Delegator",
        "Delegatee",
        "Next Delegatee(redelegate)"
    ]);
    for (let i = 0; i < planned.length; i++) {
        const tx = planned[i];
        const row = [i, tx.type, tx.quantity.toLocaleString(), tx.delegator];
        switch (tx.type) {
            case "delegate":
                row.push(tx.delegatee, "");
                break;
            case "revoke":
                row.push(tx.delegatee, "");
                break;
            case "redelegate":
                row.push(tx.prevDelegatee, tx.nextDelegatee);
                break;
            default:
                throw Error("never");
        }
        table.push(row);
    }
    const fees = new Map<string, number>();
    for (const tx of planned) {
        fees.set(tx.delegator, (fees.get(tx.delegator) || 0) + tx.fee);
    }

    console.group("Transaction plan");
    {
        console.log(table.toString());
        console.group("Total Fee");
        for (const [delegator, fee] of fees.entries()) {
            console.log(delegator, fee.toLocaleString());
        }
        console.groupEnd();
    }
    console.groupEnd();
}

async function confirm() {
    const answer = await prompts({
        type: "confirm",
        name: "value",
        message: "Execute transactions?"
    });
    if (answer.value !== true) {
        throw new Error("Cancelled");
    }
}

async function check(sdk: SDK, planned: Tx[]) {
    // all delegatees are candidates.
    const candidates = new Set(
        (await stake.getCandidates(sdk)).map(x =>
            PlatformAddress.fromPublic(x.pubkey, {
                networkId: sdk.networkId
            }).toString()
        )
    );
    for (const tx of planned) {
        let delegatee;
        if (tx.type === "delegate" || tx.type === "revoke") {
            delegatee = tx.delegatee;
        } else {
            delegatee = tx.nextDelegatee;
        }
        if (!candidates.has(delegatee)) {
            throw new Error(`Delegatee is not a candidate: ${delegatee}`);
        }
    }

    // You have enough fees.
    const fees = new Map();
    for (const tx of planned) {
        fees.set(tx.delegator, (fees.get(tx.delegator) || 0) + tx.fee);
    }
    for (const [delegator, fee] of fees) {
        const balance = await sdk.rpc.chain.getBalance(delegator);
        if (balance < fee) {
            throw new Error(
                `Stakeholder ${delegator} doesn't have enough CCC ${fee}`
            );
        }
    }
}

async function execute(
    sdk: SDK,
    passwords: Map<string, string>,
    stakeholders: PlatformAddress[],
    planned: Tx[],
    dryRun: boolean
) {
    const seqs = new Map<string, number>(
        await Promise.all(
            stakeholders.map<Promise<[string, number]>>(async s => [
                s.toString(),
                await sdk.rpc.chain.getSeq(s)
            ])
        )
    );

    const txes = [];
    for (const x of planned) {
        let tx;
        switch (x.type) {
            case "delegate":
                tx = stake.createDelegateCCSTransaction(
                    sdk,
                    x.delegatee,
                    x.quantity
                );
                break;
            case "revoke":
                tx = stake.createRevokeTransaction(
                    sdk,
                    x.delegatee,
                    x.quantity
                );
                break;
            case "redelegate":
                tx = stake.createRedelegateTransaction(
                    sdk,
                    x.prevDelegatee,
                    x.nextDelegatee,
                    x.quantity
                );
                break;
            default:
                throw new Error("never");
        }
        try {
            const passphrase = passwords.get(x.delegator);
            const seq = seqs.get(x.delegator)!;
            seqs.set(x.delegator, seq + 1);
            const signedTx = await sdk.key.signTransaction(tx, {
                account: x.delegator,
                passphrase,
                seq,
                fee: x.fee
            });
            txes.push(signedTx);
        } catch (e) {
            throw new Error(`${e}, key: ${x.delegator.toString()}`);
        }
    }
    const txHashes = txes.map(tx => tx.hash().toString());

    // Send parallel, collect results
    const results = new Map<string, TxResult>();
    if (dryRun) {
        for (const tx of txes) {
            const hash = tx.hash().toString();
            results.set(hash, {
                success: "skipped",
                hash
            });
        }
    } else {
        await Promise.all(
            txes.map(async tx => {
                const result = await sendTx(sdk, tx);
                if (result) {
                    results.set(result.hash, result);
                }
            })
        );
        await Promise.all(
            txHashes
                .filter(hash => !results.has(hash))
                .map(async hash => {
                    const result = await getResult(sdk, hash.toString());
                    results.set(result.hash, result);
                })
        );
    }

    const table = createTable(["Index", "TxHash", "Success", "Error?"]);
    for (let i = 0; i < txHashes.length; i++) {
        const result = results.get(txHashes[i].toString())!;
        table.push([i, result.hash, result.success, result.error || ""]);
    }
    console.group("Transaction result");
    console.log(table.toString());
    console.groupEnd();
    console.log("ccstake show");
}

interface TxResult {
    success: boolean | "skipped";
    hash: string;
    error?: string;
}

async function sendTx(
    sdk: SDK,
    tx: SignedTransaction
): Promise<TxResult | null> {
    const hash = tx.hash().toString();
    try {
        await sdk.rpc.chain.sendSignedTransaction(tx);
        return null;
    } catch (e) {
        return {
            success: false,
            hash,
            error: `Error in SendSignedTransaction ${e}`
        };
    }
}

async function getResult(
    sdk: SDK,
    hash: string,
    timeout: number = 20_000
): Promise<TxResult> {
    const start = Date.now();
    while (true) {
        if (await sdk.rpc.chain.containsTransaction(hash)) {
            return { success: true, hash };
        } else {
            const hint = await sdk.rpc.chain.getErrorHint(hash);
            if (hint !== null) {
                return {
                    success: false,
                    hash,
                    error: hint
                };
            }
        }
        if (Date.now() - start > timeout) {
            return {
                success: false,
                hash,
                error: "Timeout"
            };
        }
    }
}

class BatchDelegation {
    public static async fromFile(filename: string) {
        const file = await util.promisify(fs.readFile)(filename, "utf8");
        const json = JSON.parse(file);
        return BatchDelegation.fromJSON(json);
    }

    public static fromJSON(json: any) {
        const stakeholders: PlatformAddress[] = json.stakeholders.map(
            (x: any) => PlatformAddress.ensure(x)
        );
        checkUniqueSet(stakeholders.map(x => x.toString()));
        const fee = json.fee;
        const distributions: Distribution[] = json.distributions.map((x: any) =>
            Distribution.fromJSON(x)
        );
        checkUniqueSet(distributions.map(x => x.validator.toString()));
        return new BatchDelegation({
            stakeholders,
            fee,
            distributions
        });
    }

    public stakeholders: PlatformAddress[];
    public fee: number;
    public distributions: Distribution[];

    constructor(
        params: Pick<BatchDelegation, "stakeholders" | "fee" | "distributions">
    ) {
        this.stakeholders = params.stakeholders;
        this.fee = params.fee;
        this.distributions = params.distributions;
    }

    get validators() {
        return this.distributions.map(x => x.validator);
    }
}

function checkUniqueSet(values: string[]) {
    const set = new Set<string>();
    for (const value of values) {
        if (set.has(value)) {
            throw new Error(`Duplicated entries: ${value}`);
        }
        set.add(value);
    }
}

class Distribution {
    public static fromJSON(json: any) {
        return new Distribution({
            validator: PlatformAddress.ensure(json.validator),
            quantity: U64.ensure(json.quantity)
        });
    }

    public validator: PlatformAddress;
    public quantity: U64;

    constructor(params: Pick<Distribution, "validator" | "quantity">) {
        this.validator = params.validator;
        this.quantity = params.quantity;
    }
}

async function passwordsFromFile(filename: string) {
    const file = await util.promisify(fs.readFile)(filename, "utf8");
    const json = JSON.parse(file);
    if (!Array.isArray(json)) {
        throw new Error("PasswordFile format error");
    }
    const passwords = new Map<string, string>();
    for (const entry of json) {
        const address = PlatformAddress.ensure(entry.address);
        if (typeof entry.password !== "string") {
            throw new Error("PasswordFile password format error");
        }
        const password = entry.password;
        passwords.set(address.toString(), password);
    }
    return passwords;
}
