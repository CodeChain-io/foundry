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
import "mocha";
import { faucetAddress } from "../helper/constants";
import CodeChain from "../helper/spawn";
import { Transaction } from "foundry-rpc";

describe("Sealing test", function() {
    let node: CodeChain;

    beforeEach(async function() {
        node = new CodeChain();
        await node.start();
    });

    it("stopSealing then startSealing", async function() {
        await node.rpc.devel!.stopSealing();
        await node.sendPayTx();
        expect(await node.getBestBlockNumber()).to.equal(0);
        await node.rpc.devel!.startSealing();
        expect(await node.getBestBlockNumber()).to.equal(1);
    });

    afterEach(async function() {
        if (this.currentTest!.state === "failed") {
            node.keepLogs();
        }
        await node.clean();
    });
});

describe("Future queue", function() {
    let node: CodeChain;

    beforeEach(async function() {
        node = new CodeChain();
        await node.start();
    });

    it("all pending transactions must be mined", async function() {
        const seq = (await node.rpc.chain.getSeq({
            address: faucetAddress.toString(),
            blockNumber: null
        }))!;

        await node.sendPayTx({ seq: seq + 3 });
        expect(
            (await node.rpc.chain.getSeq({
                address: faucetAddress.toString(),
                blockNumber: null
            }))!
        ).to.equal(seq);
        await node.sendPayTx({ seq: seq + 2 });
        expect(
            (await node.rpc.chain.getSeq({
                address: faucetAddress.toString(),
                blockNumber: null
            }))!
        ).to.equal(seq);
        await node.sendPayTx({ seq: seq + 1 });
        expect(
            (await node.rpc.chain.getSeq({
                address: faucetAddress.toString(),
                blockNumber: null
            }))!
        ).to.equal(seq);
        await node.sendPayTx({ seq: seq });
        expect(
            (await node.rpc.chain.getSeq({
                address: faucetAddress.toString(),
                blockNumber: null
            }))!
        ).to.equal(seq + 4);
    });

    afterEach(async function() {
        if (this.currentTest!.state === "failed") {
            node.keepLogs();
        }
        await node.clean();
    });
});

describe("Get Pending Transaction", function() {
    let node: CodeChain;

    beforeEach(async function() {
        node = new CodeChain();
        await node.start();
    });

    it("all transactions in both queues should be included", async function() {
        await node.rpc.devel!.stopSealing();

        const sq = (await node.rpc.chain.getSeq({
            address: faucetAddress.toString()
        }))!;
        const tx = await node.sendPayTx({ seq: sq + 3 });
        const wholeTxs = await node.rpc.mempool.getPendingTransactions({
            futureIncluded: true
        });
        const trans = wholeTxs.transactions[0] as Transaction;
        expect(trans.sig).to.equal(tx.toJSON().sig);
    });

    afterEach(async function() {
        if (this.currentTest!.state === "failed") {
            node.keepLogs();
        }
        await node.clean();
    });
});

describe("Delete All Pending Transactions", function() {
    let node: CodeChain;

    beforeEach(async function() {
        node = new CodeChain();
        await node.start();
    });

    it("all pending transactions should be deleted", async function() {
        await node.rpc.devel!.stopSealing();

        const sq = (await node.rpc.chain.getSeq({
            address: faucetAddress.toString(),
            blockNumber: null
        }))!;

        await node.sendPayTx({ seq: sq + 0 }); // will be in the current queue
        await node.sendPayTx({ seq: sq + 3 }); // will be in the future queue
        await node.rpc.mempool.deleteAllPendingTransactions();

        const {
            transactions: wholeTXs
        } = await node.rpc.mempool.getPendingTransactions({
            from: null,
            to: null
        });

        expect(wholeTXs.length).to.equal(0);
    });

    afterEach(async function() {
        if (this.currentTest!.state === "failed") {
            node.keepLogs();
        }
        await node.clean();
    });
});
