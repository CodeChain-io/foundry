import { Transaction } from "../transaction/Transaction";
import { H256 } from "../H256";

export class ChangeShard {
    public shard_id: number;
    public pre_root: H256;
    public post_root: H256;

    constructor(obj: { shard_id: number, pre_root: H256, post_root: H256 }) {
        this.shard_id = obj.shard_id;
        this.pre_root = obj.pre_root;
        this.post_root = obj.post_root;
    }

    toJSON() {
        const { shard_id, pre_root, post_root } = this;
        return {
            shard_id,
            pre_root,
            post_root
        };
    }

    toEncodeObject() {
        const { shard_id, pre_root, post_root } = this;
        return [shard_id, pre_root.toEncodeObject(), post_root.toEncodeObject()];
    }
}

export class ChangeShardState {
    transactions: Transaction[];
    changes: ChangeShard[];

    constructor(input: { transactions: Transaction[] }) {
        const ZERO = new H256("0x0000000000000000000000000000000000000000000000000000000000000000")
        this.transactions = input.transactions;
        this.changes = [new ChangeShard({ shard_id: 0, pre_root: ZERO, post_root: ZERO })];
    }

    toEncodeObject(): Array<any> {
        const transactions = this.transactions.map(transaction => transaction.toEncodeObject());
        const changes = this.changes.map(c => c.toEncodeObject());
        return [1, transactions, changes];
    }

    toJSON() {
        return {
            action: "changeShardState",
            transactions: this.transactions.map(t => t.toJSON()),
            changes: this.changes.map(c => c.toJSON())
        };
    }
}
