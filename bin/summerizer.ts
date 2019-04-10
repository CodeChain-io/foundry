import { PlatformAddress, U64 } from "codechain-primitives/lib";
import { SDK } from "codechain-sdk";

import {
    Delegation,
    getCCSHolders,
    getDelegations,
    getPendingRevocations,
    getUndelegatedCCS,
    Revocation
} from "../src";

interface DelegationFrom {
    delegator: PlatformAddress;
    quantity: U64;
}

class QuantitySummerizableArray<T extends { quantity: U64 }> {
    public values: T[] = [];

    private _sum?: U64 | null = null;
    public get sum() {
        if (this._sum == null) {
            this._sum = sumU64(this.values.map(x => x.quantity));
        }
        return this._sum;
    }
}

export class AccountSummary {
    get balance() {
        return this.undelegated.plus(this.delegationsTo.sum);
    }
    public undelegated = new U64(0);
    public delegationsTo = new QuantitySummerizableArray<Delegation>();
    public delegationsFrom = new QuantitySummerizableArray<DelegationFrom>();
    public revocationsTo = new QuantitySummerizableArray<Revocation>();
    public revocationsFrom = new QuantitySummerizableArray<Revocation>();
}

export async function summarize(sdk: SDK, blockNumber: number) {
    const ccsHolders = await getCCSHolders(sdk, blockNumber);
    const allUndelegateds = await Promise.all(
        ccsHolders.map(ccsHolder =>
            getUndelegatedCCS(sdk, ccsHolder, blockNumber)
        )
    );
    const allDelegations = await Promise.all(
        ccsHolders.map(ccsHolder => getDelegations(sdk, ccsHolder, blockNumber))
    );
    const pendingRevocations = await getPendingRevocations(sdk, blockNumber);

    const aggregate: { [address: string]: AccountSummary } = {};

    for (let i = 0; i < ccsHolders.length; i++) {
        const delegator = ccsHolders[i];
        const delegations = allDelegations[i];

        const delegatorSummary =
            aggregate[delegator.value] || new AccountSummary();
        delegatorSummary.undelegated = allUndelegateds[i];
        delegatorSummary.delegationsTo.values = delegations;
        aggregate[delegator.value] = delegatorSummary;

        for (const { delegatee, quantity } of delegations) {
            const delegateeSummary =
                aggregate[delegatee.value] || new AccountSummary();
            delegateeSummary.delegationsFrom.values.push({
                delegator,
                quantity
            });
            aggregate[delegatee.value] = delegateeSummary;
        }
    }

    for (const revocation of pendingRevocations) {
        const { delegator, delegatee } = revocation;
        const delegatorSummary =
            aggregate[delegator.value] || new AccountSummary();
        const delegateeSummary =
            aggregate[delegatee.value] || new AccountSummary();
        delegatorSummary.revocationsTo.values.push(revocation);
        delegatorSummary.revocationsFrom.values.push(revocation);
        aggregate[delegator.value] = delegatorSummary;
        aggregate[delegatee.value] = delegateeSummary;
    }

    return {
        totalCCS: getTotalCCS(allUndelegateds, allDelegations),
        ccsHolders,
        get(account: PlatformAddress) {
            return aggregate[account.value];
        },
        delegations(delegator: PlatformAddress, delegatee: PlatformAddress) {
            const delegations = this.get(delegator).delegationsTo.values.filter(
                x => x.delegatee.value === delegatee.value
            );
            return sumU64(delegations.map(x => x.quantity));
        },
        revocations(delegator: PlatformAddress, delegatee: PlatformAddress) {
            const result = new QuantitySummerizableArray<Revocation>();
            result.values = this.get(delegator).revocationsTo.values.filter(
                x => x.delegatee.value === delegatee.value
            );
            return result;
        }
    };
}

function getTotalCCS(balances: U64[], allDelegations: Delegation[][]) {
    const totalBalance = sumU64(balances);
    let totalDelegations = new U64(0);
    for (const delegations of allDelegations) {
        totalDelegations = totalDelegations.plus(
            sumU64(delegations.map(x => x.quantity))
        );
    }
    return totalBalance.plus(totalDelegations);
}

export function sumU64(values: U64[]): U64 {
    return values.reduce((a, b) => a.plus(b), new U64(0));
}
