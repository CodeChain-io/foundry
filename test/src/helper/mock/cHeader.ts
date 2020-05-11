// Copyright 2018-2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.
import { H256, U256 } from "../../primitives/src";
import { blake256 } from "../../sdk/utils";

const RLP = require("rlp");
const BLAKE_NULL_RLP: H256 = new H256(
    "45b0cfc220ceec5b7c1c62c4d4193d38e4eba48e8815729ce75f9c0ab0e4c1c0"
);

export class Header {
    public static fromBytes(bytes: Buffer): Header {
        const decodedmsg = RLP.decode(bytes);
        const parentHash = new H256(decodedmsg[0].toString("hex"));
        const author = new H256(decodedmsg[1].toString("hex"));
        const stateRoot = new H256(decodedmsg[2].toString("hex"));
        const evidencesRoot = new H256(decodedmsg[3].toString("hex"));
        const transactionsRoot = new H256(decodedmsg[4].toString("hex"));
        const nextValidatorSetHash = new H256(decodedmsg[5].toString("hex"));
        const number = new U256(parseInt(decodedmsg[6].toString("hex"), 16));
        const timestamp = new U256(parseInt(decodedmsg[7].toString("hex"), 16));
        const lastCommittedValidators: string[] = decodedmsg[8];
        const extraData = decodedmsg[9];

        // Be careful of the order! Three roots have same types, so mistake on the order will not be catched by typechecker.
        const header = new Header(
            parentHash,
            timestamp,
            number,
            author,
            lastCommittedValidators,
            extraData,
            evidencesRoot,
            transactionsRoot,
            stateRoot,
            nextValidatorSetHash,
            []
        );

        for (let i = 8; i < decodedmsg.getLength(); i++) {
            header.seal.push(decodedmsg[i]);
        }

        return header;
    }

    public static default(): Header {
        return new Header(
            new H256(
                "0000000000000000000000000000000000000000000000000000000000000000"
            ),
            new U256(0),
            new U256(0),
            new H256(
                "0000000000000000000000000000000000000000000000000000000000000000"
            ),
            [],
            Buffer.alloc(0),
            BLAKE_NULL_RLP,
            BLAKE_NULL_RLP,
            BLAKE_NULL_RLP,
            BLAKE_NULL_RLP,
            []
        );
    }

    private parentHash: H256;
    private timestamp: U256;
    private number: U256;
    private author: H256;
    private lastCommittedValidators: string[];
    private extraData: Buffer;
    private evidencesRoot: H256;
    private transactionsRoot: H256;
    private stateRoot: H256;
    private nextValidatorSetHash: H256;
    private seal: any[];
    private hash: null | H256;
    private bareHash: null | H256;

    constructor(
        parentHash: H256,
        timestamp: U256,
        number: U256,
        author: H256,
        lastCommittedValidators: string[],
        extraData: Buffer,
        evidencesRoot: H256,
        transactionsRoot: H256,
        stateRoot: H256,
        nextValidatorSetHash: H256,
        seal: any[],
        hash?: H256,
        bareHash?: H256
    ) {
        this.parentHash = parentHash;
        this.timestamp = timestamp;
        this.number = number;
        this.author = author;
        this.lastCommittedValidators = lastCommittedValidators;
        this.extraData = extraData;
        this.evidencesRoot = evidencesRoot;
        this.transactionsRoot = transactionsRoot;
        this.stateRoot = stateRoot;
        this.nextValidatorSetHash = nextValidatorSetHash;
        this.seal = seal;
        this.hash = hash == null ? this.hashing() : hash;
        this.bareHash = bareHash == null ? null : bareHash;
    }

    public setParentHash(hash: H256) {
        this.parentHash = hash;
    }

    public setTimestamp(stamp: U256) {
        this.timestamp = stamp;
    }

    public setNumber(number: U256) {
        this.number = number;
    }

    public setAuthor(author: H256) {
        this.author = author;
    }

    public setExtraData(extraData: Buffer) {
        this.extraData = extraData;
    }

    public setEvidencesRoot(root: H256) {
        this.evidencesRoot = root;
    }

    public setTransactionsRoot(root: H256) {
        this.transactionsRoot = root;
    }

    public setStateRoot(root: H256) {
        this.stateRoot = root;
    }

    public setNextValidatorSetHash(root: H256) {
        this.nextValidatorSetHash = root;
    }

    public setSeal(seal: any[]) {
        this.seal = seal;
    }

    public getParentHash(): H256 | null {
        return this.parentHash;
    }

    public getHash(): H256 | null {
        return this.hash;
    }

    public getBareHash(): H256 | null {
        return this.bareHash;
    }

    public toEncodeObject(): Array<any> {
        return [
            this.parentHash.toEncodeObject(),
            this.author.toEncodeObject(),
            this.stateRoot.toEncodeObject(),
            this.evidencesRoot.toEncodeObject(),
            this.transactionsRoot.toEncodeObject(),
            this.nextValidatorSetHash.toEncodeObject(),
            this.number.toEncodeObject(),
            this.timestamp.toEncodeObject(),
            this.lastCommittedValidators,
            this.extraData
        ].concat(this.seal);
    }

    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    public hashing(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }
}
