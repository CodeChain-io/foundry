// Copyright 2019-2020 Kodebox, Inc.
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
import {
    aliceAddress,
    aliceSecret,
    carolSecret,
    faucetAddress,
    faucetSecret,
    stakeActionHandlerId,
    validator0Address
} from "../helper/constants";
import CodeChain from "../helper/spawn";
import { H256 } from "foundry-primitives";
import { blake256, getPublicFromPrivate } from "../sdk/utils";
import * as RLP from "rlp";

describe("Term change", function() {
    const chain = `${__dirname}/../scheme/solo.json`;
    let node: CodeChain;

    const approvalEncoded = (message: string, secret: string): any => {
        return [
            `0x${node.testFramework.util.signEd25519(message, secret)}`,
            H256.ensure(getPublicFromPrivate(secret)).toEncodeObject()
        ];
    };

    beforeEach(async function() {
        node = new CodeChain({
            chain,
            argv: ["--author", validator0Address.toString(), "--force-sealing"]
        });
        await node.start();

        const tx = await node.sendPayTx({
            fee: 10,
            quantity: 100_000,
            recipient: aliceAddress
        });
        expect(
            await node.rpc.chain.containsTransaction({
                transactionHash: `0x${tx.hash().toString()}`
            })
        ).be.true;
    });

    async function changeTermSeconds(metadataSeq: number, termSeconds: number) {
        const newParams = [
            0x20, // maxExtraDataSize
            "tc", // networkID
            10, // minPayCost
            10, // minCustomCost
            4194304, // maxBodySize
            16384, // snapshotPeriod
            termSeconds, // termSeconds
            10, // nominationExpiration
            10, // custodyPeriod
            30, // releasePeriod
            30, // maxNumOfValidators
            4, // minNumOfValidators
            4, // delegationThreshold
            1000, // minDeposit
            128, // maxCandidateMetadataSize
            0 // era
        ];
        const changeParams: (number | string | (number | string)[])[] = [
            0xff,
            metadataSeq,
            newParams
        ];
        const message = blake256(RLP.encode(changeParams).toString("hex"));
        changeParams.push(approvalEncoded(message, aliceSecret));
        changeParams.push(approvalEncoded(message, carolSecret));

        {
            const hash = await node.rpc.mempool.sendSignedTransaction({
                tx: node.testFramework.core
                    .createCustomTransaction({
                        handlerId: stakeActionHandlerId,
                        bytes: RLP.encode(changeParams)
                    })
                    .sign({
                        secret: faucetSecret,
                        seq: (await node.rpc.chain.getSeq({
                            address: faucetAddress.toString(),
                            blockNumber: null
                        }))!,
                        fee: 10
                    })
                    .rlpBytes()
                    .toString("hex")
            });
            expect(
                await node.rpc.chain.containsTransaction({
                    transactionHash: `${hash.toString()}`
                })
            ).be.true;
        }
    }

    it("initial term metadata", async function() {
        const params = (await node.rpc.chain.getTermMetadata({
            blockNumber: null
        }))!;
        expect(params).to.be.deep.equals([0, 0]);
    });

    async function waitForTermPeriodChange(termSeconds: number) {
        const lastBlockNumber = await node.rpc.chain.getBestBlockNumber();
        const lastBlock = (await node.rpc.chain.getBlockByNumber({
            blockNumber: lastBlockNumber
        }))!;

        let previousTs = lastBlock.timestamp;
        for (let count = 0; count < 20; count++) {
            await node.rpc.devel!.startSealing();
            const blockNumber = await node.rpc.chain.getBestBlockNumber();
            const block = (await node.rpc.chain.getBlockByNumber({
                blockNumber
            }))!;

            const currentTs = block.timestamp;
            const previousTermPeriod = Math.floor(previousTs / termSeconds);
            const currentTermPeriod = Math.floor(currentTs / termSeconds);
            if (previousTermPeriod !== currentTermPeriod) {
                return blockNumber;
            }
            previousTs = currentTs;
            await new Promise(resolve => setTimeout(resolve, 1000));
        }

        throw new Error("Timeout on waiting term period change");
    }

    it("can turn on term change", async function() {
        const TERM_SECONDS = 3;
        await changeTermSeconds(0, TERM_SECONDS);

        const blockNumber1 = await waitForTermPeriodChange(TERM_SECONDS);

        const params1 = (await node.rpc.chain.getTermMetadata({
            blockNumber: blockNumber1
        }))!;
        expect(params1).to.be.deep.equals([blockNumber1, 1]);

        const blockNumber2 = await waitForTermPeriodChange(TERM_SECONDS);

        const params2 = (await node.rpc.chain.getTermMetadata({
            blockNumber: blockNumber2
        }))!;
        expect(params2).to.be.deep.equals([blockNumber2, 2]);
    }).timeout(10_000);

    afterEach(async function() {
        if (this.currentTest!.state === "failed") {
            node.keepLogs();
        }
        await node.clean();
    });
});
