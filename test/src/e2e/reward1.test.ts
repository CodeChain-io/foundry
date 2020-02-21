// Copyright 2018-2019 Kodebox, Inc.
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
import "mocha";
import { faucetAddress } from "../helper/constants";
import CodeChain from "../helper/spawn";

describe("reward1", function() {
    let node: CodeChain;

    beforeEach(async function() {
        node = new CodeChain({
            chain: `${__dirname}/../scheme/solo-block-reward-50.json`,
            argv: ["--force-sealing"]
        });

        await node.start();
    });

    it("getBlockReward", async function() {
        const reward = await node.rpc.engine.getBlockReward({
            blockNumber: 10
        });
        expect(reward).to.equal(50);
    });

    it("null if the block is not mined", async function() {
        const bestBlockNumber = await node.rpc.chain.getBestBlockNumber();
        const nonMinedBlockNumber = bestBlockNumber + 10;
        expect(
            await node.rpc.chain.getMiningReward({
                blockNumber: nonMinedBlockNumber
            })
        ).to.equal(null);
    });

    it("mining reward of the empty block is the same with the block reward", async function() {
        await node.rpc.devel!.startSealing();
        const bestBlockNumber = await node.rpc.chain.getBestBlockNumber();
        const miningReward = await node.rpc.chain.getMiningReward({
            blockNumber: bestBlockNumber
        });
        const blockReward = +(await node.rpc.engine.getBlockReward({
            blockNumber: bestBlockNumber
        }))!;
        expect(miningReward).to.equal(blockReward);
    });

    it("mining reward includes the block fee", async function() {
        await node.rpc.devel!.stopSealing();
        const seq = (await node.rpc.chain.getSeq({
            address: faucetAddress.toString(),
            blockNumber: null
        }))!;
        await node.sendPayTx({
            quantity: 10,
            fee: 123,
            seq
        });
        await node.sendPayTx({
            quantity: 10,
            fee: 456,
            seq: seq + 1
        });
        await node.sendPayTx({
            quantity: 10,
            fee: 321,
            seq: seq + 2
        });
        await node.rpc.devel!.startSealing();
        const bestBlockNumber = await node.rpc.chain.getBestBlockNumber();
        const miningReward = await node.rpc.chain.getMiningReward({
            blockNumber: bestBlockNumber
        });
        const blockReward = +(await node.rpc.engine.getBlockReward({
            blockNumber: bestBlockNumber
        }))!;
        expect(miningReward).to.equal(blockReward + 123 + 456 + 321);
    });

    afterEach(async function() {
        if (this.currentTest!.state === "failed") {
            node.keepLogs();
        }
        await node.clean();
    });
});
