import { PlatformAddress } from "../key/PlatformAddress";
import { H256 } from "./H256";
import { U256 } from "./U256";
import { SignedParcel } from "./SignedParcel";

export type BlockData = {
    parentHash: H256;
    timestamp: number;
    number: number;
    author: PlatformAddress;
    extraData: Buffer;
    parcelsRoot: H256;
    stateRoot: H256;
    invoicesRoot: H256;
    score: U256;
    seal: Buffer[];
    hash: H256;
    parcels: SignedParcel[];
};
/**
 * Block is the unit of processes being handled by CodeChain. Contains information related to SignedParcel's list and block creation.
 */
export class Block {
    parentHash: H256;
    timestamp: number;
    number: number;
    author: PlatformAddress;
    extraData: Buffer;
    parcelsRoot: H256;
    stateRoot: H256;
    invoicesRoot: H256;
    score: U256;
    seal: Buffer[];
    hash: H256;
    parcels: SignedParcel[];

    constructor(data: BlockData) {
        const { parentHash, timestamp, number, author, extraData,
            parcelsRoot, stateRoot, invoicesRoot, score, seal, hash, parcels } = data;
        this.parentHash = parentHash;
        this.timestamp = timestamp;
        this.number = number;
        this.author = author;
        this.extraData = extraData;
        this.parcelsRoot = parcelsRoot;
        this.stateRoot = stateRoot;
        this.invoicesRoot = invoicesRoot;
        this.score = score;
        this.seal = seal;
        this.hash = hash;
        this.parcels = parcels;
    }

    static fromJSON(data: any) {
        const { parentHash, timestamp, number, author, extraData,
            parcelsRoot, stateRoot, invoicesRoot, score, seal, hash, parcels } = data;
        return new this({
            parentHash: new H256(parentHash),
            timestamp,
            number,
            author: PlatformAddress.fromString(author),
            extraData,
            parcelsRoot: new H256(parcelsRoot),
            stateRoot: new H256(stateRoot),
            invoicesRoot: new H256(invoicesRoot),
            score: new U256(score),
            seal,
            hash: new H256(hash),
            parcels: parcels.map((p: any) => SignedParcel.fromJSON(p))
        });
    }

    toJSON() {
        const { parentHash, timestamp, number, author, extraData,
            parcelsRoot, stateRoot, invoicesRoot, score, seal, hash, parcels } = this;
        return {
            parentHash: parentHash.value,
            timestamp,
            number,
            author: author.toString(),
            extraData,
            parcelsRoot: parcelsRoot.value,
            stateRoot: stateRoot.value,
            invoicesRoot: invoicesRoot.value,
            score: score.value.toString(),
            seal,
            hash: hash.value,
            parcels: parcels.map(p => p.toJSON()),
        };
    }
}
