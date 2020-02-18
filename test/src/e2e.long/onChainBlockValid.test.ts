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
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

import { expect } from "chai";
import { H160, H256, PlatformAddress, U256 } from "codechain-primitives/lib";
import "mocha";
import { Mock } from "../helper/mock";
import { Header } from "../helper/mock/cHeader";
import CodeChain from "../helper/spawn";

describe("Test onChain block communication", async function() {
    let nodeA: CodeChain;
    let mock: Mock;
    let soloGenesisBlock: Header;
    let soloBlock1: Header;
    let soloBlock2: Header;

    let VALID_PARENT = new H256(
        "ff8324bd3b0232e4fd1799496ae422ee0896cc7a8a64a2885052e320b4ba9535"
    );
    let VALID_AUTHOR = new H160("7777777777777777777777777777777777777777");
    let VALID_TRANSACTIONS_ROOT = new H256(
        "45b0cfc220ceec5b7c1c62c4d4193d38e4eba48e8815729ce75f9c0ab0e4c1c0"
    );
    let VALID_STATEROOT = new H256(
        "2f6b19afc38f6f1464af20dde08d8bebd6a6aec0a95aaf7ef2fb729c3b88dc5b"
    );

    before(async function() {
        const node = new CodeChain({
            argv: ["--force-sealing"]
        });
        await node.start();

        const rpc = node.rpc;

        await rpc.devel!.startSealing();
        await rpc.devel!.startSealing();

        const genesisBlock = await rpc.chain.getBlockByNumber({
            blockNumber: 0
        });
        if (genesisBlock == null) {
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

        await node.clean();
        const author1PlatformAddr = PlatformAddress.fromString(
            genesisBlock.author
        );
        soloGenesisBlock = new Header(
            new H256(genesisBlock.parentHash),
            new U256(genesisBlock.timestamp),
            new U256(genesisBlock.number),
            author1PlatformAddr.accountId,
            Buffer.from(genesisBlock.extraData),
            new H256(genesisBlock.transactionsRoot),
            new H256(genesisBlock.stateRoot),
            new H256(genesisBlock.nextValidatorSetHash),
            new U256(`${genesisBlock.score}`),
            genesisBlock.seal
        );
        const author2PlatformAddr = PlatformAddress.fromString(block1.author);
        soloBlock1 = new Header(
            soloGenesisBlock.hashing(),
            new U256(block1.timestamp),
            new U256(block1.number),
            author2PlatformAddr.accountId,
            Buffer.from(block1.extraData),
            new H256(block1.transactionsRoot),
            new H256(block1.stateRoot),
            new H256(block1.nextValidatorSetHash),
            new U256(2222222222222),
            block1.seal
        );
        const author3PlatformAddr = PlatformAddress.fromString(block2.author);
        soloBlock2 = new Header(
            soloBlock1.hashing(),
            new U256(block2.timestamp),
            new U256(block2.number),
            author3PlatformAddr.accountId,
            Buffer.from(block2.extraData),
            new H256(block2.transactionsRoot),
            new H256(block2.stateRoot),
            new H256(block2.nextValidatorSetHash),
            new U256(33333333333333),
            block2.seal
        );

        VALID_PARENT = new H256(block1.parentHash);
        VALID_AUTHOR = author2PlatformAddr.accountId;
        VALID_TRANSACTIONS_ROOT = new H256(block1.transactionsRoot);
        VALID_STATEROOT = new H256(block1.stateRoot);

        nodeA = new CodeChain();
        await nodeA.start();
        mock = new Mock("0.0.0.0", nodeA.port, "tc");
        await mock.establish();
    });

    it("OnChain valid block propagation test", async function() {
        // mock.setLog();
        const rpc = nodeA.rpc;

        // Genesis block
        const header = soloGenesisBlock;

        // Block 1
        const header1 = soloBlock1;

        // Block 2
        const header2 = soloBlock2;

        await mock.sendEncodedBlock(
            [
                header.toEncodeObject(),
                header1.toEncodeObject(),
                header2.toEncodeObject()
            ],
            [[], []],
            header2.hashing(),
            header2.getScore()
        );

        await mock.waitStatusMessage();

        const block1 = await rpc.chain.getBlockByNumber({ blockNumber: 1 });
        const block2 = await rpc.chain.getBlockByNumber({ blockNumber: 2 });

        expect(block1).not.to.be.null;
        expect(block2).not.to.be.null;
    }).timeout(50_000);

    afterEach(async function() {
        if (this.currentTest!.state === "failed") {
            nodeA.keepLogs();
        }
        await mock.end();
        await nodeA.clean();
    });
});
