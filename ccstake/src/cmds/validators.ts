import { PlatformAddress } from "codechain-sdk/lib/core/classes";
import {
    Candidate,
    getBanned,
    getCandidates,
    getJailed,
    getTermMetadata,
    getValidators,
    Validator
} from "codechain-stakeholder-sdk";
import stripAnsi from "strip-ansi";
import * as yargs from "yargs";

import { GlobalParams } from "..";
import { summarize } from "../summerizer";
import { asyncHandler, createTable, prologue } from "../util";

export const module: yargs.CommandModule<GlobalParams, GlobalParams> = {
    command: "validators",
    describe: "Show validators",
    handler: asyncHandler(async argv => {
        const { sdk, blockNumber } = await prologue(argv);

        const [
            termMetadata,
            validators,
            candidates,
            jailed,
            banned
        ] = await Promise.all([
            getTermMetadata(sdk, blockNumber),
            getValidators(sdk, blockNumber),
            getCandidates(sdk, blockNumber),
            getJailed(sdk, blockNumber),
            getBanned(sdk, blockNumber)
        ]);
        const summary = await summarize(sdk, blockNumber);

        console.log("Current Term:", termMetadata!.currentTermId);
        console.log(
            "Last term finished block:",
            termMetadata!.lastTermFinishedBlockNumber
        );

        console.group("Validators:");
        {
            const table = createTable([
                "Address",
                "Delegation (CCS)",
                "Deposit (CCC)"
            ]);
            const desc = (a: Validator, b: Validator) => {
                if (a.delegation.gt(b.delegation)) {
                    return -1;
                }
                if (a.delegation.lt(b.delegation)) {
                    return 1;
                }
                if (a.deposit.gt(b.deposit)) {
                    return -1;
                }
                if (a.deposit.lt(b.deposit)) {
                    return 1;
                }
                return 0;
            };
            validators.sort(desc);
            for (const { pubkey, delegation, deposit } of validators) {
                table.push([
                    PlatformAddress.fromPublic(pubkey, {
                        networkId: sdk.networkId
                    }).toString(),
                    delegation.toLocaleString(),
                    deposit.toLocaleString()
                ]);
            }
            console.log(table.toString());
        }
        console.groupEnd();

        console.group("Candidates: ");
        {
            const sortedCandidats = candidates.map(c => {
                const address = PlatformAddress.fromPublic(c.pubkey, {
                    networkId: sdk.networkId
                });
                const delegation = summary.get(address).delegationsFrom.sum;
                return { ...c, address, delegation };
            });
            const desc = (
                a: typeof sortedCandidats[0],
                b: typeof sortedCandidats[0]
            ) => {
                if (a.delegation.gt(b.delegation)) {
                    return -1;
                }
                if (a.delegation.lt(b.delegation)) {
                    return 1;
                }
                if (a.deposit.gt(b.deposit)) {
                    return -1;
                }
                if (a.deposit.lt(b.deposit)) {
                    return 1;
                }
                return 0;
            };
            sortedCandidats.sort(desc);
            const table = createTable([
                "Address",
                "Delegation (CCS)",
                "Deposit (CCC)",
                "Ends at (Term)",
                "Metadata"
            ]);
            for (const {
                address,
                delegation,
                deposit,
                nominationEndsAt,
                metadata
            } of sortedCandidats) {
                table.push([
                    address.toString(),
                    delegation.toLocaleString(),
                    deposit.toLocaleString(),
                    nominationEndsAt.toString(),
                    stripAnsi(metadata.toString())
                ]);
            }
            console.log(table.toString());
        }
        console.groupEnd();

        console.group("Jailed: ");
        {
            const table = createTable([
                "Address",
                "Deposit (CCC)",
                "Custody until(Term)",
                "Ends at (Term)"
            ]);
            for (const {
                address,
                deposit,
                custodyUntil,
                releasedAt
            } of jailed) {
                table.push([
                    address.toString(),
                    deposit.toLocaleString(),
                    custodyUntil.toString(),
                    releasedAt.toString()
                ]);
            }
            console.log(table.toString());
        }
        console.groupEnd();

        console.group("Banned: ");
        {
            const table = createTable(["Address"]);
            for (const address of banned) {
                table.push([address.toString()]);
            }
            console.log(table.toString());
        }
        console.groupEnd();
    })
};
