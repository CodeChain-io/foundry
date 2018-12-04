import { Transaction } from "../transaction/Transaction";

export class AssetTransaction {
    public transaction: Transaction;
    public approvals: string[];

    constructor(input: { transaction: Transaction; approvals: string[] }) {
        this.transaction = input.transaction;
        this.approvals = input.approvals;
    }

    public toEncodeObject(): any[] {
        const transaction = this.transaction.toEncodeObject();
        const approvals = this.approvals;
        return [1, transaction, approvals];
    }

    public toJSON() {
        return {
            action: "assetTransaction",
            transaction: this.transaction.toJSON(),
            approvals: this.approvals
        };
    }
}
