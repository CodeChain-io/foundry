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
import { wait } from "../helper/promise";
import CodeChain from "../helper/spawn";
import { SignedTransaction } from "../sdk/src/core/classes";

describe("Memory pool size test", function() {
    let nodeA: CodeChain;
    const sizeLimit: number = 4;

    beforeEach(async function() {
        nodeA = new CodeChain({
            argv: ["--mem-pool-size", sizeLimit.toString()]
        });
        await nodeA.start();
        await nodeA.rpc.devel!.stopSealing();
    });

    it("To self", async function() {
        const sending = [];
        for (let seq = 0; seq < sizeLimit * 2; seq++) {
            sending.push(nodeA.sendPayTx({ seq }));
        }
        await Promise.all(sending);
        const pendingTransactions = await nodeA.rpc.mempool.getPendingTransactions();
        expect(pendingTransactions.transactions.length).to.equal(sizeLimit * 2);
    }).timeout(10_000);

    describe("To others", async function() {
        let nodeB: CodeChain;

        beforeEach(async function() {
            nodeB = new CodeChain({
                argv: [
                    "--mem-pool-size",
                    sizeLimit.toString(),
                    "--bootstrap-addresses",
                    `127.0.0.1:${nodeA.port}`
                ]
            });
            await nodeB.start();
            await nodeB.rpc.devel!.stopSealing();
        });

        it("More than limit", async function() {
            for (let seq = 0; seq < sizeLimit * 2; seq++) {
                await nodeA.sendPayTx({
                    seq
                });
            }

            while (
                (await nodeB.rpc.mempool.getPendingTransactions()).transactions
                    .length < sizeLimit
            ) {
                await wait(500);
            }

            const pendingTransactions = await nodeB.rpc.mempool.getPendingTransactions();
            expect(pendingTransactions.transactions.length).to.equal(sizeLimit);
        }).timeout(60_000);

        it("Rejected by limit and reaccepted", async function() {
            const sent = [];
            for (let seq = 0; seq < sizeLimit * 2; seq++) {
                sent.push(
                    await nodeA.sendPayTx({
                        seq
                    })
                );
            }

            while (
                (await nodeB.rpc.mempool.getPendingTransactions()).transactions
                    .length < sizeLimit
            ) {
                await wait(500);
            }

            const pendingTransactions = await nodeB.rpc.mempool.getPendingTransactions();
            const pendingTransactionHashes = pendingTransactions.transactions.map(
                (tx: any) => tx.hash
            );
            const rejectedTransactions = sent.filter(
                tx => !pendingTransactionHashes.includes(`0x${tx.hash().value}`)
            );

            await nodeB.rpc.devel!.startSealing();

            while (
                (await nodeB.rpc.mempool.getPendingTransactions()).transactions
                    .length > 0
            ) {
                await wait(500);
            }

            await nodeB.rpc.devel!.stopSealing();

            await Promise.all(
                rejectedTransactions.map((tx: SignedTransaction) =>
                    nodeB.rpc.mempool
                        .sendSignedTransaction({
                            tx: `0x${tx.rlpBytes().toString("hex")}`
                        })
                        .then(txhash =>
                            expect(txhash).to.eq(`0x${tx.hash().value}`)
                        )
                )
            );

            const pendingTransactionsAfterResend = await nodeB.rpc.mempool.getPendingTransactions();
            const pendingTransactionHashesAfterResend = pendingTransactionsAfterResend.transactions.map(
                (tx: any) => tx.hash
            );

            rejectedTransactions.forEach(
                tx =>
                    expect(
                        pendingTransactionHashesAfterResend.includes(
                            `0x${tx.hash().value}`
                        )
                    ).to.true
            );
        }).timeout(60_000);

        afterEach(async function() {
            await nodeB.clean();
        });
    }).timeout(60_000);

    afterEach(async function() {
        await nodeA.clean();
    });
});

// FIXME: The size of Pay transaction is about 100 byte.
// To exceed 1 MB limit, we need to send more than 10,000 pay transactions. It takes too much time.
// Let's enable this test again after introducing a transaction whose size is large.
describe.skip("Memory pool memory limit test", function() {
    let nodeA: CodeChain;
    const memoryLimit: number = 1;

    beforeEach(async function() {
        nodeA = new CodeChain({
            chain: `${__dirname}/../scheme/mempool.json`,
            argv: ["--mem-pool-mem-limit", memoryLimit.toString()]
        });
        await nodeA.start();
        await nodeA.rpc.devel!.stopSealing();
    });

    it("To self", async function() {
        let remainSize = memoryLimit * 1024 * 1024;
        const bigEnough = 1024 * 1024;
        const txs = [];
        for (let i = 0; i < bigEnough; i++) {
            const tx = nodeA.createPayTx({ seq: i });
            remainSize -= tx.rlpBytes().byteLength;
            const trans = tx.rlpBytes().toString("hex");
            txs.push(
                nodeA.rpc.mempool.sendSignedTransaction({ tx: `0x${trans}` })
            );

            if (remainSize < 0) {
                break;
            }
        }
        await Promise.all(txs);
        const pendingTransactions = await nodeA.rpc.mempool.getPendingTransactions();
        expect(pendingTransactions.transactions.length).to.equal(txs.length);
    }).timeout(50_000);

    describe("To others", async function() {
        let nodeB: CodeChain;

        beforeEach(async function() {
            this.timeout(60_000);
            nodeB = new CodeChain({
                chain: `${__dirname}/../scheme/mempool.json`,
                argv: ["--mem-pool-mem-limit", memoryLimit.toString()]
            });
            await nodeB.start();
            await nodeB.rpc.devel!.stopSealing();

            await nodeA.connect(nodeB);
        });

        it("More than limit", async function() {
            const [aBlockNumber, bBlockNumber] = await Promise.all([
                nodeA.rpc.chain.getBestBlockNumber(),
                nodeB.rpc.chain.getBestBlockNumber()
            ]);
            expect(aBlockNumber).to.equal(bBlockNumber);
            let remainSize = memoryLimit * 1024 * 1024;
            const bigEnough = 1024 * 1024;
            const txs = [];
            for (let i = 0; i < bigEnough; i++) {
                const tx = nodeA.createPayTx({ seq: i });
                remainSize -= tx.rlpBytes().byteLength;
                const trans = tx.rlpBytes().toString("hex");
                txs.push(
                    nodeA.rpc.mempool.sendSignedTransaction({
                        tx: `0x${trans}`
                    })
                );

                if (remainSize < 0) {
                    break;
                }
            }
            await Promise.all(txs);
            await wait(3_000);

            const pendingTransactions = await nodeB.rpc.mempool.getPendingTransactions();
            expect(pendingTransactions.transactions.length).to.equal(
                txs.length - 1
            );
            expect(await nodeA.rpc.chain.getBestBlockNumber()).to.equal(
                aBlockNumber
            );
            expect(await nodeB.rpc.chain.getBestBlockNumber()).to.equal(
                bBlockNumber
            );
        }).timeout(60_000);

        afterEach(async function() {
            await nodeB.clean();
        });
    });

    afterEach(async function() {
        await nodeA.clean();
    });
});
