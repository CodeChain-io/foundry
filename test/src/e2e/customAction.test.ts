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

import { fail } from "assert";
import * as chai from "chai";
import * as chaiAsPromised from "chai-as-promised";
import { toHex } from "foundry-primitives/lib";
import "mocha";
import {
    faucetAddress,
    faucetSecret,
    hitActionHandlerId
} from "../helper/constants";
import { ERROR } from "../helper/error";
import CodeChain from "../helper/spawn";
import * as RLP from "rlp";

chai.use(chaiAsPromised);
const expect = chai.expect;

const hitcount = toHex(RLP.encode(["hit count"]));
const closecount = toHex(RLP.encode(["close count"]));
const nonexistingkey = toHex(RLP.encode(["non-existing-key"]));
describe("customAction", function() {
    let node: CodeChain;

    before(async function() {
        node = new CodeChain();
        await node.start();
    });

    describe("customAction", function() {
        it("should get initial state", async function() {
            const actionData = await node.rpc.engine.getCustomActionData({
                handlerId: hitActionHandlerId,
                bytes: `0x${hitcount}`,
                blockNumber: null
            });

            expect(actionData).to.be.equal(toHex(RLP.encode(1)));
        });

        it("should alter state", async function() {
            const previousHitData = (await node.rpc.engine.getCustomActionData({
                handlerId: hitActionHandlerId,
                bytes: `0x${hitcount}`,
                blockNumber: null
            }))!;
            const previousHitCount = Buffer.from(
                previousHitData,
                "hex"
            ).readUInt8(0);

            const previousBlockCloseData = (await node.rpc.engine.getCustomActionData(
                {
                    handlerId: hitActionHandlerId,
                    bytes: `0x${closecount}`,
                    blockNumber: null
                }
            ))!;
            const previousBlockCloseCount = Buffer.from(
                previousBlockCloseData,
                "hex"
            ).readUInt8(0);

            const increment = 11;
            const tx = node.testFramework.core
                .createCustomTransaction({
                    handlerId: hitActionHandlerId,
                    bytes: RLP.encode([increment])
                })
                .sign({
                    secret: faucetSecret,
                    seq: (await node.rpc.chain.getSeq({
                        address: faucetAddress.toString(),
                        blockNumber: null
                    }))!,
                    fee: 10
                });
            const trans = tx.rlpBytes().toString("hex");
            const hash = await node.rpc.mempool.sendSignedTransaction({
                tx: `0x${trans}`
            });
            expect(
                await node.rpc.chain.containsTransaction({
                    transactionHash: hash
                })
            ).be.true;
            expect(
                await node.rpc.chain.getTransaction({ transactionHash: hash })
            ).not.null;

            const hitData = (await node.rpc.engine.getCustomActionData({
                handlerId: hitActionHandlerId,
                bytes: `0x${hitcount}`,
                blockNumber: null
            }))!;

            expect(hitData).to.be.equal(
                toHex(RLP.encode(previousHitCount + increment))
            );
            const closeData = (await node.rpc.engine.getCustomActionData({
                handlerId: hitActionHandlerId,
                bytes: `0x${closecount}`,
                blockNumber: null
            }))!;
            expect(closeData).to.be.equal(
                toHex(RLP.encode(previousBlockCloseCount + 1))
            );
        });

        it("should return null", async function() {
            const actionData = await node.rpc.engine.getCustomActionData({
                handlerId: hitActionHandlerId,
                bytes: `0x${nonexistingkey}`,
                blockNumber: null
            });

            expect(actionData).to.be.null;
        });

        it("should throw state not exist", async function() {
            try {
                await node.rpc.engine.getCustomActionData({
                    handlerId: hitActionHandlerId,
                    bytes: `0x${hitcount}`,
                    blockNumber: 99999
                });
                fail();
            } catch (e) {
                expect(e.toString()).include(ERROR.STATE_NOT_EXIST);
            }
        });

        it("should throw handler not found on getCustomActionData", async function() {
            try {
                await node.rpc.engine.getCustomActionData({
                    handlerId: 999999,
                    bytes: `0x${toHex(RLP.encode([]))}`,
                    blockNumber: null
                });
                fail();
            } catch (e) {
                expect(e.toString()).include(
                    ERROR.ACTION_DATA_HANDLER_NOT_FOUND
                );
            }
        });

        it("should throw handler not found on sendCustomTransaction", async function() {
            try {
                const tx = node.testFramework.core
                    .createCustomTransaction({
                        handlerId: 99999,
                        bytes: RLP.encode([11])
                    })
                    .sign({
                        secret: faucetSecret,
                        seq: (await node.rpc.chain.getSeq({
                            address: faucetAddress.toString(),
                            blockNumber: null
                        }))!,
                        fee: 10
                    });
                const trans = tx.rlpBytes().toString("hex");
                await node.rpc.mempool.sendSignedTransaction({
                    tx: `0x${trans}`
                });
                fail();
            } catch (e) {
                expect(e.toString()).include(
                    ERROR.ACTION_DATA_HANDLER_NOT_FOUND
                );
            }
        });

        it("should fail on handling error", async function() {
            const seq = (await node.rpc.chain.getSeq({
                address: faucetAddress.toString(),
                blockNumber: null
            }))!;
            const blockNumber = await node.rpc.chain.getBestBlockNumber();
            const tx = node.testFramework.core
                .createCustomTransaction({
                    handlerId: hitActionHandlerId,
                    bytes: RLP.encode(["wrong", "format", "of", "message"])
                })
                .sign({
                    secret: faucetSecret,
                    seq: seq + 1,
                    fee: 10
                });
            const trans = tx.rlpBytes().toString("hex");
            expect(node.rpc.mempool.sendSignedTransaction({ tx: `0x${trans}` }))
                .be.rejected;
        });
    });

    afterEach(function() {
        if (this.currentTest!.state === "failed") {
            node.keepLogs();
        }
    });

    after(async function() {
        await node.clean();
    });
});
