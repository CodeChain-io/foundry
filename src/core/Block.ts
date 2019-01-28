import { H256, PlatformAddress, U256 } from "codechain-primitives";

import { SignedTransaction } from "./SignedTransaction";
import { fromJSONToSignedTransaction } from "./transaction/json";

// Disable lint error from using "number" as variable name
// tslint:disable:variable-name

export interface BlockData {
    parentHash: H256;
    timestamp: number;
    number: number;
    author: PlatformAddress;
    extraData: Buffer;
    transactionsRoot: H256;
    stateRoot: H256;
    invoicesRoot: H256;
    score: U256;
    seal: Buffer[];
    hash: H256;
    transactions: SignedTransaction[];
}
/**
 * Block is the unit of processes being handled by CodeChain. Contains information related to SignedTransaction's list and block creation.
 */
export class Block {
    public static fromJSON(data: any) {
        const {
            parentHash,
            timestamp,
            number,
            author,
            extraData,
            transactionsRoot,
            stateRoot,
            invoicesRoot,
            score,
            seal,
            hash,
            transactions
        } = data;
        return new this({
            parentHash: new H256(parentHash),
            timestamp,
            number,
            author: PlatformAddress.fromString(author),
            extraData,
            transactionsRoot: new H256(transactionsRoot),
            stateRoot: new H256(stateRoot),
            invoicesRoot: new H256(invoicesRoot),
            score: new U256(score),
            seal,
            hash: new H256(hash),
            transactions: transactions.map(fromJSONToSignedTransaction)
        });
    }
    public parentHash: H256;
    public timestamp: number;
    public number: number;
    public author: PlatformAddress;
    public extraData: Buffer;
    public transactionsRoot: H256;
    public stateRoot: H256;
    public invoicesRoot: H256;
    public score: U256;
    public seal: Buffer[];
    public hash: H256;
    public transactions: SignedTransaction[];

    constructor(data: BlockData) {
        const {
            parentHash,
            timestamp,
            number,
            author,
            extraData,
            transactionsRoot,
            stateRoot,
            invoicesRoot,
            score,
            seal,
            hash,
            transactions
        } = data;
        this.parentHash = parentHash;
        this.timestamp = timestamp;
        this.number = number;
        this.author = author;
        this.extraData = extraData;
        this.transactionsRoot = transactionsRoot;
        this.stateRoot = stateRoot;
        this.invoicesRoot = invoicesRoot;
        this.score = score;
        this.seal = seal;
        this.hash = hash;
        this.transactions = transactions;
    }

    public toJSON() {
        const {
            parentHash,
            timestamp,
            number,
            author,
            extraData,
            transactionsRoot,
            stateRoot,
            invoicesRoot,
            score,
            seal,
            hash,
            transactions
        } = this;
        return {
            parentHash: parentHash.toJSON(),
            timestamp,
            number,
            author: author.toString(),
            extraData,
            transactionsRoot: transactionsRoot.toJSON(),
            stateRoot: stateRoot.toJSON(),
            invoicesRoot: invoicesRoot.toJSON(),
            score: score.value.toString(),
            seal,
            hash: hash.toJSON(),
            transactions: transactions.map(p => p.toJSON())
        };
    }
}
