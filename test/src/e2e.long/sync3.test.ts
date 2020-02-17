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
import CodeChain from "../helper/spawn";

describe("sync 3 nodes", function() {
    const NUM_NODES = 3;
    let nodes: CodeChain[];

    beforeEach(async function() {
        this.timeout(15_000 + 10_000 * NUM_NODES);

        nodes = [];
        for (let i = 0; i < NUM_NODES; i++) {
            const node = new CodeChain({
                argv: ["--no-discovery"]
            });
            nodes.push(node);
        }
        await Promise.all(nodes.map(node => node.start()));
    });

    describe("Connected in a line", function() {
        describe("All connected", function() {
            beforeEach(async function() {
                this.timeout(15_000 + 10_000 * NUM_NODES);

                const connects = [];
                for (let i = 0; i < NUM_NODES - 1; i++) {
                    connects.push(nodes[i].connect(nodes[i + 1]));
                }
                await Promise.all(connects);
            });

            it("It should be synced when the first node created a block", async function() {
                const blockNumber = await nodes[0].getBestBlockNumber();
                const payTx = await nodes[0].sendPayTx();
                expect(
                    await nodes[0].rpc.chain.containsTransaction({
                        transactionHash: `0x${payTx.hash().toString()}`
                    })
                ).be.true;
                const transaction = (await nodes[0].rpc.chain.getTransaction({
                    transactionHash: `0x${payTx.hash().toString()}`
                }))!;
                expect(transaction).not.null;
                await nodes[0].waitBlockNumber(blockNumber + 1);
                for (let i = 1; i < NUM_NODES; i++) {
                    await nodes[i].waitBlockNumberSync(nodes[i - 1]);
                    expect(
                        (await nodes[i].getBestBlockHash()).toString()
                    ).to.deep.equal(transaction.blockHash!.substr(2));
                }
            }).timeout(15_000 + 10_000 * NUM_NODES);
        });

        describe("the first node becomes ahead", function() {
            beforeEach(async function() {
                await nodes[0].sendPayTx();
            });

            it("It should be synced when every node connected", async function() {
                for (let i = 0; i < NUM_NODES - 1; i++) {
                    await nodes[i].connect(nodes[i + 1]);
                    await nodes[i + 1].waitBlockNumberSync(nodes[i]);
                    expect(await nodes[i].getBestBlockHash()).to.deep.equal(
                        await nodes[i + 1].getBestBlockHash()
                    );
                }
            }).timeout(15_000 + 15_000 * NUM_NODES);
        });
    }).timeout(NUM_NODES * 60_000);

    describe("Connected in a circle", function() {
        const numHalf: number = Math.floor(NUM_NODES / 2);

        beforeEach(async function() {
            this.timeout(15_000 + 10_000 * NUM_NODES);

            const connects = [];
            for (let i = 0; i < NUM_NODES; i++) {
                connects.push(nodes[i].connect(nodes[(i + 1) % NUM_NODES]));
            }
            await Promise.all(connects);
        });

        it("It should be synced when the first node created a block", async function() {
            const payTx = await nodes[0].sendPayTx();
            expect(
                await nodes[0].rpc.chain.containsTransaction({
                    transactionHash: `0x${payTx.hash().toString()}`
                })
            ).be.true;
            const transaction = (await nodes[0].rpc.chain.getTransaction({
                transactionHash: `0x${payTx.hash().toString()}`
            }))!;
            expect(transaction).not.null;
            for (let i = 1; i <= numHalf; i++) {
                await nodes[0].waitBlockNumberSync(nodes[i]);
                expect(
                    (await nodes[i].getBestBlockHash()).toString()
                ).to.deep.equal(transaction.blockHash!.substr(2));

                await nodes[0].waitBlockNumberSync(nodes[NUM_NODES - i - 1]);
                expect(
                    (
                        await nodes[NUM_NODES - i - 1].getBestBlockHash()
                    ).toString()
                ).to.deep.equal(transaction.blockHash!.substr(2));
            }
        }).timeout(15_000 + 10_000 * NUM_NODES);
    }).timeout(NUM_NODES * 60_000);

    afterEach(async function() {
        if (this.currentTest!.state === "failed") {
            nodes.map(node => node.keepLogs());
        }

        await Promise.all(nodes.map(node => node.clean()));
        nodes = [];
    });
});
