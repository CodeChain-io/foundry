import { Buffer } from "buffer";
import { expect } from "chai";
import "mocha";
import * as RLP from "rlp";
import {
    getPublicFromPrivate,
    H256,
    signEd25519,
    U256
} from "../../../primitives/src";
import { blake256 } from "../../../primitives/src/hash";
import { Header } from "../cHeader";

describe("Check Header RLP encoding", function() {
    it("empty Header RLP encoding test", function() {
        const header = Header.default();
        // Find the empty header's rlp encoded data in the unit test in header.rs file
        expect(header.rlpBytes().toString("hex")).deep.equal(
            "f8caa00000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000a045b0cfc220ceec5b7c1c62c4d4193d38e4eba48e8815729ce75f9c0ab0e4c1c0a045b0cfc220ceec5b7c1c62c4d4193d38e4eba48e8815729ce75f9c0ab0e4c1c0a045b0cfc220ceec5b7c1c62c4d4193d38e4eba48e8815729ce75f9c0ab0e4c1c0a045b0cfc220ceec5b7c1c62c4d4193d38e4eba48e8815729ce75f9c0ab0e4c1c08080c080"
        );
    });

    it("Header RLP encoding test", function() {
        const privateKey =
            "5a0391789b130315eebeb333d4fa641aee07242081ba8858ed3f36a937ca84653b21399e52ae4d7582032df537c00eaa3f4611210b3305ce48ac5407cd8f91bf";
        const publicKey = getPublicFromPrivate(privateKey);
        const header = Header.default();
        header.setNumber(new U256(4));
        header.setAuthor(new H256(publicKey));
        const bitset = Buffer.alloc(100, 0);
        bitset[0] = 4;
        const signature = createPrecommit({
            height: 3,
            view: 0,
            step: 2,
            parentHash: header.getParentHash()!,
            privateKey
        });
        header.setSeal([0, 0, [Buffer.from(signature, "hex")], bitset]);
        // Find the header's rlp encoded data in the unit test in the tendermint/mod.rs file
        expect(header.rlpBytes().toString("hex")).deep.equal(
            "f90176a00000000000000000000000000000000000000000000000000000000000000000a03b21399e52ae4d7582032df537c00eaa3f4611210b3305ce48ac5407cd8f91bfa045b0cfc220ceec5b7c1c62c4d4193d38e4eba48e8815729ce75f9c0ab0e4c1c0a045b0cfc220ceec5b7c1c62c4d4193d38e4eba48e8815729ce75f9c0ab0e4c1c0a045b0cfc220ceec5b7c1c62c4d4193d38e4eba48e8815729ce75f9c0ab0e4c1c0a045b0cfc220ceec5b7c1c62c4d4193d38e4eba48e8815729ce75f9c0ab0e4c1c00480c0808080f842b8404752ae46a97e2e4ff11b8212b85610f81ae63c5b541cc5f1e89238150c122be650b9abc954bab919de4be9a2f1dba992e88b9aa5596d2bdf2645597163697d07b86404000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        );
    });

    function createPrecommit({
        height,
        view,
        step,
        parentHash,
        privateKey
    }: {
        height: number;
        view: number;
        step: number;
        parentHash: H256;
        privateKey: string;
    }): string {
        const voteOn = [[height, view, step], [parentHash.toEncodeObject()]];
        const serializedVoteOn = RLP.encode(voteOn);
        const message = blake256(serializedVoteOn);
        return signEd25519(message, privateKey);
    }
});
