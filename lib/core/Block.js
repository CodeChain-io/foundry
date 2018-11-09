"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const codechain_primitives_1 = require("codechain-primitives");
const H256_1 = require("./H256");
const SignedParcel_1 = require("./SignedParcel");
const U256_1 = require("./U256");
/**
 * Block is the unit of processes being handled by CodeChain. Contains information related to SignedParcel's list and block creation.
 */
class Block {
    static fromJSON(data) {
        const { parentHash, timestamp, number, author, extraData, parcelsRoot, stateRoot, invoicesRoot, score, seal, hash, parcels } = data;
        return new this({
            parentHash: new H256_1.H256(parentHash),
            timestamp,
            number,
            author: codechain_primitives_1.PlatformAddress.fromString(author),
            extraData,
            parcelsRoot: new H256_1.H256(parcelsRoot),
            stateRoot: new H256_1.H256(stateRoot),
            invoicesRoot: new H256_1.H256(invoicesRoot),
            score: new U256_1.U256(score),
            seal,
            hash: new H256_1.H256(hash),
            parcels: parcels.map((p) => SignedParcel_1.SignedParcel.fromJSON(p))
        });
    }
    constructor(data) {
        const { parentHash, timestamp, number, author, extraData, parcelsRoot, stateRoot, invoicesRoot, score, seal, hash, parcels } = data;
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
    toJSON() {
        const { parentHash, timestamp, number, author, extraData, parcelsRoot, stateRoot, invoicesRoot, score, seal, hash, parcels } = this;
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
            parcels: parcels.map(p => p.toJSON())
        };
    }
}
exports.Block = Block;
