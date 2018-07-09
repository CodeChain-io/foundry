import { Transaction } from "../Transaction";

export class ChangeShardState {
    transactions: Transaction[];

    constructor(transactions: Transaction[]) {
        this.transactions = transactions;
    }

    toEncodeObject(): Array<any> {
        return [1, this.transactions.map(transaction => transaction.toEncodeObject())];
    }

    toJSON() {
        return {
            action: "changeShardState",
            transactions: this.transactions.map(t => t.toJSON())
        };
    }
}
