// Copyright 2018-2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

import { expect } from "chai";
import { Block } from "foundry-rpc";
import "mocha";
import Test = Mocha.Test;
import { Mock } from "../helper/mock/";
import { Header } from "../helper/mock/cHeader";
import CodeChain from "../helper/spawn";
import { Address, H256, U256 } from "../primitives/src";

const BLAKE_NULL_RLP: H256 = new H256(
    "45b0cfc220ceec5b7c1c62c4d4193d38e4eba48e8815729ce75f9c0ab0e4c1c0"
);
async function setup(): Promise<[Header, Block, Header]> {
    const temporaryNode = new CodeChain({
        argv: ["--force-sealing"]
    });
    await temporaryNode.start();

    const rpc = temporaryNode.rpc;

    await rpc.devel!.startSealing();
    await rpc.devel!.startSealing();

    const block0 = await rpc.chain.getBlockByNumber({ blockNumber: 0 });
    if (block0 == null) {
        throw Error("Cannot get the genesis block");
    }
    const block1 = await rpc.chain.getBlockByNumber({ blockNumber: 1 });
    if (block1 == null) {
        throw Error("Cannot get the first block");
    }
    const block2 = await rpc.chain.getBlockByNumber({ blockNumber: 2 });
    if (block2 == null) {
        throw Error("Cannot get the second block");
    }

    await temporaryNode.clean();
    const authorPaddress = Address.fromString(block0.author);
    const header0 = new Header(
        new H256(block0.parentHash),
        new U256(block0.timestamp),
        new U256(block0.number),
        authorPaddress.pubkey,
        [], // lastCommittedValidators
        Buffer.from(block0.extraData),
        BLAKE_NULL_RLP,
        new H256(block0.transactionsRoot),
        new H256(block0.nextValidatorSetHash),
        new H256(block0.stateRoot),
        block0.seal
    );
    const author1 = Address.fromString(block1.author);
    const header1 = new Header(
        header0.hashing(),
        new U256(block1.timestamp),
        new U256(block1.number),
        author1.pubkey,
        [], // lastCommittedValidators
        Buffer.from(block1.extraData),
        BLAKE_NULL_RLP,
        new H256(block1.transactionsRoot),
        new H256(block1.nextValidatorSetHash),
        new H256(block1.stateRoot),
        block1.seal
    );
    const author3 = Address.fromString(block0.author);
    const header2 = new Header(
        header1.hashing(),
        new U256(block2.timestamp),
        new U256(block2.number),
        author3.pubkey,
        [], // lastCommittedValidators
        Buffer.from(block2.extraData),
        BLAKE_NULL_RLP,
        new H256(block2.transactionsRoot),
        new H256(block2.nextValidatorSetHash),
        new H256(block2.stateRoot),
        block2.seal
    );
    return [header0, block1, header2];
}

async function setupEach(): Promise<[CodeChain, Mock]> {
    const node = new CodeChain();
    await node.start();
    const mock = new Mock("0.0.0.0", node.port, "tc");
    await mock.establish();
    return [node, mock];
}

async function teardownEach(currentTest: Test, mock: Mock, node: CodeChain) {
    if (currentTest.state === "failed") {
        node.keepLogs();
    }
    await mock.end();
    await node.clean();
}

async function testBody(
    header0: Header,
    block1: Block,
    header2: Header,
    mock: Mock,
    params: {
        tparent?: H256;
        ttimeStamp?: U256;
        tnumber?: U256;
        tauthor?: H256;
        textraData?: Buffer;
        evidencesRoot?: H256;
        ttransactionRoot?: H256;
        tstateRoot?: H256;
        tnextValidatorSetHash?: H256;
        tseal?: number[][];
    }
) {
    const {
        tnumber,
        textraData,
        tparent,
        tauthor,
        evidencesRoot,
        ttransactionRoot,
        tstateRoot,
        tnextValidatorSetHash,
        tseal
    } = params;

    const bestHash = header2.hashing();

    const author4 = Address.fromString(block1.author);
    const header = new Header(
        header0.hashing(),
        new U256(block1.timestamp),
        new U256(block1.number),
        author4.pubkey,
        [], // lastCommittedValidators
        Buffer.from(block1.extraData),
        BLAKE_NULL_RLP,
        new H256(block1.transactionsRoot),
        new H256(block1.nextValidatorSetHash),
        new H256(block1.stateRoot),
        block1.seal
    );

    if (tparent != null) {
        header.setParentHash(tparent);
    }
    if (tnumber != null) {
        header.setNumber(tnumber);
    }
    if (tauthor != null) {
        header.setAuthor(tauthor);
    }
    if (textraData != null) {
        header.setExtraData(textraData);
    }
    if (evidencesRoot != null) {
        header.setEvidencesRoot(evidencesRoot);
    }
    if (ttransactionRoot != null) {
        header.setTransactionsRoot(ttransactionRoot);
    }
    if (tstateRoot != null) {
        header.setStateRoot(tstateRoot);
    }
    if (tnextValidatorSetHash != null) {
        header.setNextValidatorSetHash(tnextValidatorSetHash);
    }
    if (tseal != null) {
        header.setSeal(tseal);
    }

    const genesis = mock.genesisHash;
    const seq = new U256(0);
    await mock.sendStatus(seq, bestHash, genesis);
    await mock.sendBlockHeaderResponse([
        [header0.toEncodeObject(), []],
        [header.toEncodeObject(), []],
        // [[]] => empty validator set
        [header2.toEncodeObject(), [[]]]
    ]);
    await mock.waitHeaderRequest();

    const bodyRequest = mock.getBlockBodyRequest();

    expect(bodyRequest).to.be.null;
}

export function createTestSuite(
    testNumber: number,
    title: string,
    params: any
) {
    // tslint:disable only-arrow-functions
    describe(`invalid block propagation ${testNumber}`, async function() {
        this.timeout(60_000);
        // tslint:enable only-arrow-functions
        let node: CodeChain;
        let mock: Mock;
        let header0: Header;
        let block1: Block;
        let header2: Header;

        // tslint:disable only-arrow-functions
        before(async function() {
            // tslint:enable only-arrow-functions
            [header0, block1, header2] = await setup();
        });

        // tslint:disable only-arrow-functions
        beforeEach(async function() {
            // tslint:enable only-arrow-functions
            [node, mock] = await setupEach();
        });

        afterEach(async function() {
            await teardownEach(this.currentTest!, mock, node);
        });

        // tslint:disable only-arrow-functions
        it(title, async function() {
            // tslint:enable only-arrow-functions
            await testBody(header0, block1, header2, mock, params);
        }).timeout(30_000);
    });
}
