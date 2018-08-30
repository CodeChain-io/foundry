import { H256 } from "../H256";
import { Transaction } from "../transaction/Transaction";

export class ChangeShard {
    public shardId: number;
    public preRoot: H256;
    public postRoot: H256;

    constructor(obj: { shardId: number; preRoot: H256; postRoot: H256 }) {
        this.shardId = obj.shardId;
        this.preRoot = obj.preRoot;
        this.postRoot = obj.postRoot;
    }

    public toJSON() {
        const { shardId, preRoot, postRoot } = this;
        return {
            shardId,
            preRoot: preRoot.toEncodeObject(),
            postRoot: postRoot.toEncodeObject()
        };
    }

    public toEncodeObject() {
        const { shardId, preRoot, postRoot } = this;
        return [shardId, preRoot.toEncodeObject(), postRoot.toEncodeObject()];
    }
}

export class ChangeShardState {
    public transactions: Transaction[];
    public changes: ChangeShard[];
    public signatures: string[];

    constructor(input: { transactions: Transaction[] }) {
        const ZERO = new H256(
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );
        this.transactions = input.transactions;
        this.changes = [
            new ChangeShard({ shardId: 0, preRoot: ZERO, postRoot: ZERO })
        ];
        this.signatures = [];
    }

    public addSignature(signature: string) {
        this.signatures.push(signature);
    }

    public toEncodeObject(): any[] {
        const transactions = this.transactions.map(transaction =>
            transaction.toEncodeObject()
        );
        const changes = this.changes.map(c => c.toEncodeObject());
        const signatures = this.signatures;
        return [1, transactions, changes, signatures];
    }

    public toJSON() {
        return {
            action: "changeShardState",
            transactions: this.transactions.map(t => t.toJSON()),
            changes: this.changes.map(c => c.toJSON()),
            signatures: this.signatures
        };
    }
}
