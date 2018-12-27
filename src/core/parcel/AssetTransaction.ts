import { Transaction } from "../classes";
import { Parcel } from "../Parcel";
import { NetworkId } from "../types";

export class AssetTransaction extends Parcel {
    private readonly _transaction: Transaction;
    private readonly approvals: string[];
    public constructor(
        input: { transaction: Transaction; approvals: string[] },
        networkId: NetworkId
    ) {
        super(networkId);

        this._transaction = input.transaction;
        this.approvals = input.approvals;
    }

    public transaction(): Transaction {
        return this._transaction;
    }

    protected actionToEncodeObject(): any[] {
        const transaction = this._transaction.toEncodeObject();
        const approvals = this.approvals;
        return [1, transaction, approvals];
    }

    protected actionToJSON(): any {
        return {
            transaction: this._transaction.toJSON(),
            approvals: this.approvals
        };
    }

    protected action(): string {
        return "assetTransaction";
    }
}
