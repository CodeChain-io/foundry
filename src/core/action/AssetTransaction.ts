import { Transaction } from "../transaction/Transaction";

export class AssetTransaction {
    public transaction: Transaction;

    constructor(input: { transaction: Transaction }) {
        this.transaction = input.transaction;
    }

    public toEncodeObject(): any[] {
        const transaction = this.transaction.toEncodeObject();
        return [1, transaction];
    }

    public toJSON() {
        return {
            action: "assetTransaction",
            transaction: this.transaction.toJSON()
        };
    }
}
