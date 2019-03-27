import { H256, PlatformAddress, U256 } from "codechain-primitives";

import { SignedTransaction, SignedTransactionJSON } from "./SignedTransaction";
import { fromJSONToSignedTransaction } from "./transaction/json";

// Disable lint error from using "number" as variable name
// tslint:disable:variable-name

export interface BlockData {
    parentHash: H256;
    timestamp: number;
    number: number;
    author: PlatformAddress;
    extraData: number[];
    transactionsRoot: H256;
    stateRoot: H256;
    score: U256;
    seal: number[][];
    hash: H256;
    transactions: SignedTransaction[];
}
export interface BlockJSON {
    parentHash: string;
    timestamp: number;
    number: number;
    author: string;
    extraData: number[];
    transactionsRoot: string;
    stateRoot: string;
    score: string;
    seal: number[][];
    hash: string;
    transactions: SignedTransactionJSON[];
}
/**
 * Block is the unit of processes being handled by CodeChain. Contains information related to SignedTransaction's list and block creation.
 */
export class Block {
    public static fromJSON(data: BlockJSON) {
        const {
            parentHash,
            timestamp,
            number,
            author,
            extraData,
            transactionsRoot,
            stateRoot,
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
    public extraData: number[];
    public transactionsRoot: H256;
    public stateRoot: H256;
    public score: U256;
    public seal: number[][];
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
        this.score = score;
        this.seal = seal;
        this.hash = hash;
        this.transactions = transactions;
    }

    public toJSON(): BlockJSON {
        const {
            parentHash,
            timestamp,
            number,
            author,
            extraData,
            transactionsRoot,
            stateRoot,
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
            extraData: [...extraData],
            transactionsRoot: transactionsRoot.toJSON(),
            stateRoot: stateRoot.toJSON(),
            score: score.value.toString(),
            seal: seal.map(buffer => [...buffer]),
            hash: hash.toJSON(),
            transactions: transactions.map(p => p.toJSON())
        };
    }
}
