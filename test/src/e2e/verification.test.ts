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
import { faucetAddress, faucetSecret } from "../helper/constants";
import { ERROR } from "../helper/error";
import CodeChain from "../helper/spawn";
import * as RLP from "rlp";

describe("solo - 1 node", function() {
    const recipient = "tccqxv9y4cw0jwphhu65tn4605wadyd2sxu5yezqghw";

    let node: CodeChain;
    before(async function() {
        node = new CodeChain();
        await node.start();
    });

    describe("Sending invalid transactions over the limits (general)", function() {
        let encoded: any[];
        beforeEach(async function() {
            const seq = (await node.rpc.chain.getSeq({
                address: faucetAddress.toString(),
                blockNumber: null
            }))!;
            const tx = node.testFramework.core
                .createPayTransaction({
                    recipient,
                    quantity: 0
                })
                .sign({
                    secret: faucetSecret,
                    fee: 10,
                    seq
                });
            encoded = tx.toEncodeObject();
        });

        ["0x01" + "0".repeat(64), "0x" + "f".repeat(128)].forEach(function(
            seq
        ) {
            it(`seq: ${seq}`, async function() {
                encoded[0] = seq;
                try {
                    await node.sendSignedTransactionWithRlpBytes(
                        RLP.encode(encoded)
                    );
                    expect.fail();
                } catch (e) {
                    expect(e).is.similarTo(ERROR.INVALID_RLP_TOO_BIG);
                }
            });
        });

        ["0x01" + "0".repeat(64), "0x" + "f".repeat(128)].forEach(function(
            fee
        ) {
            it(`fee: ${fee}`, async function() {
                encoded[1] = fee;
                try {
                    await node.sendSignedTransactionWithRlpBytes(
                        RLP.encode(encoded)
                    );
                    expect.fail();
                } catch (e) {
                    expect(e).is.similarTo(ERROR.INVALID_RLP_TOO_BIG);
                }
            });
        });

        ["tcc", "a", "ac"].forEach(function(networkId) {
            it(`networkId: ${networkId}`, async function() {
                encoded = [];
                {
                    // Since Foundry checks signature before checking the network id,
                    // we should sign the transaction with invalid network id.
                    const seq = (await node.rpc.chain.getSeq({
                        address: faucetAddress.toString(),
                        blockNumber: null
                    }))!;
                    const unsigned = node.testFramework.core.createPayTransaction(
                        {
                            recipient,
                            quantity: 0
                        }
                    );
                    (unsigned as any)._networkId = networkId;

                    const tx = unsigned.sign({
                        secret: faucetSecret,
                        fee: 10,
                        seq
                    });
                    encoded = tx.toEncodeObject();
                }

                expect(encoded[2]).equals(networkId);

                try {
                    await node.sendSignedTransactionWithRlpBytes(
                        RLP.encode(encoded)
                    );
                    expect.fail();
                } catch (e) {
                    if (networkId.length !== 2)
                        expect(e).is.similarTo(
                            ERROR.INVALID_RLP_INVALID_LENGTH
                        );
                    else expect(e).is.similarTo(ERROR.INVALID_NETWORK_ID);
                }
            });
        });

        [0, 10, 100].forEach(function(action) {
            it(`action (invalid type): ${action}`, async function() {
                encoded[3] = [action];
                try {
                    await node.sendSignedTransactionWithRlpBytes(
                        RLP.encode(encoded)
                    );
                    expect.fail();
                } catch (e) {
                    expect(e).is.similarTo(
                        ERROR.INVALID_RLP_UNEXPECTED_ACTION_PREFIX
                    );
                }
            });
        });

        [
            { actionType: 2, actionLength: 2 }, // Pay
            { actionType: 2, actionLength: 4 },
            { actionType: 5, actionLength: 2 }, // SetShardOwners
            { actionType: 5, actionLength: 4 },
            { actionType: 6, actionLength: 2 }, // SetShardUsers
            { actionType: 6, actionLength: 4 },
            { actionType: 0x19, actionLength: 3 }, // ShardStore
            { actionType: 0x19, actionLength: 5 }
        ].forEach(function(params: {
            actionType: number;
            actionLength: number;
        }) {
            const { actionType, actionLength } = params;
            it(`action (type / invalid length): ${actionType}, ${actionLength}`, async function() {
                encoded[3] = Array(actionLength).fill(actionType);
                try {
                    await node.sendSignedTransactionWithRlpBytes(
                        RLP.encode(encoded)
                    );
                    expect.fail();
                } catch (e) {
                    expect(e).is.similarTo(
                        ERROR.INVALID_RLP_INCORRECT_LIST_LEN
                    );
                }
            });
        });

        [
            "0x00",
            "0x1" + "0".repeat(125),
            "0x1" + "0".repeat(128),
            "0x" + "f".repeat(129)
        ].forEach(function(sig) {
            it(`signature: ${sig}`, async function() {
                encoded[4] = sig;
                try {
                    await node.sendSignedTransactionWithRlpBytes(
                        RLP.encode(encoded)
                    );
                    expect.fail();
                } catch (e) {
                    expect(e).is.similarTo(ERROR.INVALID_RLP_INVALID_LENGTH);
                }
            });
        });
    });

    describe("Sending invalid transactions over the limits (in action 2: Pay)", function() {
        let encoded: any[];
        beforeEach(async function() {
            const seq = (await node.rpc.chain.getSeq({
                address: faucetAddress.toString(),
                blockNumber: null
            }))!;
            const signed = node.testFramework.core
                .createPayTransaction({
                    recipient,
                    quantity: 0
                })
                .sign({
                    secret: faucetSecret,
                    fee: 10,
                    seq
                });
            encoded = signed.toEncodeObject();
        });

        ["0x1" + "0".repeat(40), "0x" + "f".repeat(38)].forEach(function(
            recipient
        ) {
            it(`recipient: ${recipient}`, async function() {
                encoded[3][1] = recipient;
                try {
                    await node.sendSignedTransactionWithRlpBytes(
                        RLP.encode(encoded)
                    );
                    expect.fail();
                } catch (e) {
                    if (recipient.length < 42)
                        expect(e).is.similarTo(ERROR.INVALID_RLP_TOO_SHORT);
                    else expect(e).is.similarTo(ERROR.INVALID_RLP_TOO_BIG);
                }
            });
        });

        ["0x01" + "0".repeat(64), "0x" + "f".repeat(128)].forEach(function(
            amount
        ) {
            it(`amount: ${amount}`, async function() {
                encoded[3][2] = amount;
                try {
                    await node.sendSignedTransactionWithRlpBytes(
                        RLP.encode(encoded)
                    );
                    expect.fail();
                } catch (e) {
                    expect(e).is.similarTo(ERROR.INVALID_RLP_TOO_BIG);
                }
            });
        });
    });

    describe("Sending invalid transactions over the limits (in action 5: SetShardOwners)", function() {
        let encoded: any[];
        beforeEach(async function() {
            const seq = (await node.rpc.chain.getSeq({
                address: faucetAddress.toString(),
                blockNumber: null
            }))!;
            const account = await node.createaddress();
            const signed = node.testFramework.core
                .createSetShardOwnersTransaction({
                    shardId: 0,
                    owners: [account]
                })
                .sign({
                    secret: faucetSecret,
                    fee: 10,
                    seq
                });
            encoded = signed.toEncodeObject();
        });

        [65536, 100000].forEach(function(shardId) {
            it(`shardId: ${shardId}`, async function() {
                encoded[3][1] = shardId;
                try {
                    await node.sendSignedTransactionWithRlpBytes(
                        RLP.encode(encoded)
                    );
                    expect.fail();
                } catch (e) {
                    expect(e).is.similarTo(ERROR.INVALID_RLP_TOO_BIG);
                }
            });
        });

        it("Owners");
    });

    describe("Sending invalid transactions over the limits (in action 6: SetShardUsers)", function() {
        let encoded: any[];
        beforeEach(async function() {
            const seq = (await node.rpc.chain.getSeq({
                address: faucetAddress.toString(),
                blockNumber: null
            }))!;
            const account = await node.createaddress();
            const signed = node.testFramework.core
                .createSetShardUsersTransaction({
                    shardId: 0,
                    users: [account]
                })
                .sign({
                    secret: faucetSecret,
                    fee: 10,
                    seq
                });
            encoded = signed.toEncodeObject();
        });

        [65536, 100000].forEach(function(shardId) {
            it(`shardId: ${shardId}`, async function() {
                encoded[3][1] = shardId;
                try {
                    await node.sendSignedTransactionWithRlpBytes(
                        RLP.encode(encoded)
                    );
                    expect.fail();
                } catch (e) {
                    expect(e).is.similarTo(ERROR.INVALID_RLP_TOO_BIG);
                }
            });
        });

        it("Users");
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
