import * as RLP from "rlp";
import { Address, H256 } from "../../primitives/src";
import { SignedTransaction, SignedTransactionJSON } from "./SignedTransaction";
import { fromJSONToSignedTransaction } from "./transaction/json";

// Disable lint error from using "number" as variable name
// tslint:disable:variable-name

export interface BlockData {
    parentHash: H256;
    timestamp: number;
    number: number;
    author: Address;
    extraData: number[];
    transactionsRoot: H256;
    stateRoot: H256;
    nextValidatorSetHash: H256;
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
    nextValidatorSetHash: string;
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
            nextValidatorSetHash,
            seal,
            hash,
            transactions
        } = data;
        return new this({
            parentHash: new H256(parentHash),
            timestamp,
            number,
            author: Address.fromString(author),
            extraData,
            transactionsRoot: new H256(transactionsRoot),
            stateRoot: new H256(stateRoot),
            nextValidatorSetHash: new H256(nextValidatorSetHash),
            seal,
            hash: new H256(hash),
            transactions: transactions.map(fromJSONToSignedTransaction)
        });
    }
    public parentHash: H256;
    public timestamp: number;
    public number: number;
    public author: Address;
    public extraData: number[];
    public transactionsRoot: H256;
    public stateRoot: H256;
    public nextValidatorSetHash: H256;
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
            nextValidatorSetHash,
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
        this.nextValidatorSetHash = nextValidatorSetHash;
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
            nextValidatorSetHash,
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
            nextValidatorSetHash: nextValidatorSetHash.toJSON(),
            seal: seal.map(buffer => [...buffer]),
            hash: hash.toJSON(),
            transactions: transactions.map(p => p.toJSON())
        };
    }

    public getSize(): number {
        const {
            parentHash,
            timestamp,
            number,
            author,
            extraData,
            transactionsRoot,
            stateRoot,
            nextValidatorSetHash,
            seal,
            transactions
        } = this;

        const blockHeader: any[] = [];
        blockHeader.push(parentHash.toEncodeObject());
        blockHeader.push(author.getPubKey().toEncodeObject());
        blockHeader.push(stateRoot.toEncodeObject());
        blockHeader.push(transactionsRoot.toEncodeObject());
        blockHeader.push(nextValidatorSetHash.toEncodeObject());
        blockHeader.push(number);
        blockHeader.push(timestamp);
        blockHeader.push(`0x${Buffer.from(extraData).toString("hex")}`);
        blockHeader.push(
            ...seal.map(s => `0x${Buffer.from(s).toString("hex")}`)
        );

        const encoded: Buffer = RLP.encode([
            blockHeader,
            [], // evidences
            transactions.map(tx => tx.toEncodeObject())
        ]);

        return encoded.byteLength;
    }
}
