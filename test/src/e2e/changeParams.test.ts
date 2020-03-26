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

import * as chai from "chai";
import { expect } from "chai";
import * as chaiAsPromised from "chai-as-promised";
import { H256, Address } from "foundry-primitives";
import "mocha";
import {
    aliceAddress,
    aliceSecret,
    bobSecret,
    carolAddress,
    carolSecret,
    faucetAddress,
    faucetSecret,
    stakeActionHandlerId,
    validator0Address
} from "../helper/constants";
import CodeChain from "../helper/spawn";
import { blake256, getPublicFromPrivate } from "../sdk/utils";
import { ERROR } from "../helper/error";
import * as RLP from "rlp";

chai.use(chaiAsPromised);

describe("ChangeParams", function() {
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
                transactionHash: "0x".concat(tx.hash().toString())
            })
        ).be.true;
    });

    it("change", async function() {
        const newParams = [
            0x20, // maxExtraDataSize
            "tc", // networkID
            11, // minPayCost
            10, // minCustomCost
            4194304, // maxBodySize
            16384, // snapshotPeriod
            0, // termSeconds
            100, // nominationExpiration
            100, // custodyPeriod
            200, // releasePeriod
            10, // maxNumOfValidators
            1, // minNumOfValidators
            100, // delegationThreshold
            100, // minDeposit
            500, // maxCandidateMetadataSize
            0 // era
        ];
        const changeParams: (number | string | (number | string)[])[] = [
            0xff,
            0,
            newParams
        ];
        const message = blake256(RLP.encode(changeParams).toString("hex"));
        changeParams.push(approvalEncoded(message, aliceSecret));
        changeParams.push(approvalEncoded(message, carolSecret));

        {
            const tx = node.testFramework.core
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
                });
            const trans = tx.rlpBytes().toString("hex");
            const hash = await node.rpc.mempool.sendSignedTransaction({
                tx: trans
            });
            expect(
                await node.rpc.chain.containsTransaction({
                    transactionHash: hash
                })
            ).be.true;
        }
        try {
            await node.sendPayTx({ fee: 10 });
            expect.fail();
        } catch (e) {
            expect(e).is.similarTo(ERROR.TOO_LOW_FEE);
        }
        const params = await node.rpc.chain.getCommonParams({});
        expect(+params!.minPayCost!).to.be.deep.equal(11);
    });

    it("cannot change the network id", async function() {
        const newParams = [
            0x20, // maxExtraDataSize
            "cc", // networkID
            11, // minPayCost
            10, // minCustomCost
            4194304, // maxBodySize
            16384, // snapshotPeriod
            0, // termSeconds
            100, // nominationExpiration
            100, // custodyPeriod
            200, // releasePeriod
            10, // maxNumOfValidators
            1, // minNumOfValidators
            100, // delegationThreshold
            100, // minDeposit
            500, // maxCandidateMetadataSize
            0 // era
        ];
        const changeParams: (number | string | (number | string)[])[] = [
            0xff,
            0,
            newParams
        ];
        const message = blake256(RLP.encode(changeParams).toString("hex"));
        changeParams.push(approvalEncoded(message, aliceSecret));
        changeParams.push(approvalEncoded(message, carolSecret));

        const tx = node.testFramework.core
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
            });
        const trans = tx.rlpBytes().toString("hex");
        try {
            await node.rpc.mempool.sendSignedTransaction({ tx: `0x${trans}` });
            expect.fail();
        } catch (e) {
            expect(e).is.similarTo(ERROR.ACTION_DATA_HANDLER_NOT_FOUND);
        }
    });

    it("should keep default common params value", async function() {
        const params = await node.rpc.chain.getCommonParams({
            blockNumber: null
        });
        expect(+params!.minPayCost!).to.be.deep.equal(10);
    });

    it("the parameter is applied from the next block", async function() {
        const newParams = [
            0x20, // maxExtraDataSize
            "tc", // networkID
            11, // minPayCost
            10, // minCustomCost
            4194304, // maxBodySize
            16384, // snapshotPeriod
            0, // termSeconds
            100, // nominationExpiration
            100, // custodyPeriod
            200, // releasePeriod
            10, // maxNumOfValidators
            1, // minNumOfValidators
            100, // delegationThreshold
            100, // minDeposit
            500, // maxCandidateMetadataSize
            0 // era
        ];
        const changeParams: (number | string | (number | string)[])[] = [
            0xff,
            0,
            newParams
        ];
        const message = blake256(RLP.encode(changeParams).toString("hex"));
        changeParams.push(approvalEncoded(message, aliceSecret));
        changeParams.push(approvalEncoded(message, bobSecret));
        changeParams.push(approvalEncoded(message, carolSecret));

        {
            await node.rpc.devel!.stopSealing();
            const blockNumber = await node.rpc.chain.getBestBlockNumber();
            const seq = (await node.rpc.chain.getSeq({
                address: faucetAddress.toString(),
                blockNumber: null
            }))!;
            const tx = node.testFramework.core
                .createCustomTransaction({
                    handlerId: stakeActionHandlerId,
                    bytes: RLP.encode(changeParams)
                })
                .sign({
                    secret: faucetSecret,
                    seq,
                    fee: 10
                });
            const trans = tx.rlpBytes().toString("hex");
            const changeHash = await node.rpc.mempool.sendSignedTransaction({
                tx: `0x${trans}`
            });
            const pay = await node.sendPayTx({ seq: seq + 1, fee: 10 });
            await node.rpc.devel!.startSealing();
            expect(
                await node.rpc.chain.containsTransaction({
                    transactionHash: changeHash
                })
            ).be.true;
            expect(
                await node.rpc.chain.containsTransaction({
                    transactionHash: "0x".concat(pay.hash().toString())
                })
            ).be.true;
            expect(await node.rpc.chain.getBestBlockNumber()).equal(
                blockNumber + 1
            );
        }
        try {
            await node.sendPayTx({ fee: 10 });
            expect.fail();
        } catch (e) {
            expect(e).is.similarTo(ERROR.TOO_LOW_FEE);
        }
    });

    it("the parameter changed twice in the same block", async function() {
        const newParams1 = [
            0x20, // maxExtraDataSize
            "tc", // networkID
            11, // minPayCost
            10, // minCustomCost
            4194304, // maxBodySize
            16384, // snapshotPeriod
            0, // termSeconds
            100, // nominationExpiration
            100, // custodyPeriod
            200, // releasePeriod
            10, // maxNumOfValidators
            1, // minNumOfValidators
            100, // delegationThreshold
            100, // minDeposit
            500, // maxCandidateMetadataSize
            0 // era
        ];
        const newParams2 = [
            0x20, // maxExtraDataSize
            "tc", // networkID
            5, // minPayCost
            10, // minCustomCost
            4194304, // maxBodySize
            16384, // snapshotPeriod
            0, // termSeconds
            100, // nominationExpiration
            100, // custodyPeriod
            200, // releasePeriod
            10, // maxNumOfValidators
            1, // minNumOfValidators
            100, // delegationThreshold
            100, // minDeposit
            500, // maxCandidateMetadataSize
            0 // era
        ];
        const changeParams1: (number | string | (number | string)[])[] = [
            0xff,
            0,
            newParams1
        ];
        const changeParams2: (number | string | (number | string)[])[] = [
            0xff,
            1,
            newParams2
        ];
        const message1 = blake256(RLP.encode(changeParams1).toString("hex"));
        changeParams1.push(approvalEncoded(message1, aliceSecret));
        changeParams1.push(approvalEncoded(message1, bobSecret));
        changeParams1.push(approvalEncoded(message1, carolSecret));
        const message2 = blake256(RLP.encode(changeParams2).toString("hex"));
        changeParams2.push(approvalEncoded(message2, aliceSecret));
        changeParams2.push(approvalEncoded(message2, bobSecret));
        changeParams2.push(approvalEncoded(message2, carolSecret));

        {
            await node.rpc.devel!.stopSealing();
            const blockNumber = await node.rpc.chain.getBestBlockNumber();
            const seq = (await node.rpc.chain.getSeq({
                address: faucetAddress.toString(),
                blockNumber: null
            }))!;

            const tx = node.testFramework.core
                .createCustomTransaction({
                    handlerId: stakeActionHandlerId,
                    bytes: RLP.encode(changeParams1)
                })
                .sign({
                    secret: faucetSecret,
                    seq,
                    fee: 10
                });
            const trans = tx.rlpBytes().toString("hex");
            const changeHash1 = await node.rpc.mempool.sendSignedTransaction({
                tx: `0x${trans}`
            });
            const tx2 = node.testFramework.core
                .createCustomTransaction({
                    handlerId: stakeActionHandlerId,
                    bytes: RLP.encode(changeParams2)
                })
                .sign({
                    secret: faucetSecret,
                    seq: seq + 1,
                    fee: 10
                });
            const trans2 = tx2.rlpBytes().toString("hex");
            const changeHash2 = await node.rpc.mempool.sendSignedTransaction({
                tx: `0x${trans2}`
            });
            await node.rpc.devel!.startSealing();
            expect(
                await node.rpc.chain.containsTransaction({
                    transactionHash: changeHash1
                })
            ).be.true;
            expect(
                await node.rpc.chain.containsTransaction({
                    transactionHash: changeHash2
                })
            ).be.true;
            expect(await node.rpc.chain.getBestBlockNumber()).equal(
                blockNumber + 1
            );
        }

        const pay = await node.sendPayTx({ fee: 5 });
        expect(
            await node.rpc.chain.containsTransaction({
                transactionHash: `0x${pay.hash().toString()}`
            })
        ).be.true;
        try {
            await node.sendPayTx({ fee: 4 });
            expect.fail();
        } catch (e) {
            expect(e).is.similarTo(ERROR.TOO_LOW_FEE);
        }
    });

    it("cannot reuse the same signature", async function() {
        const newParams1 = [
            0x20, // maxExtraDataSize
            "tc", // networkID
            11, // minPayCost
            10, // minCustomCost
            4194304, // maxBodySize
            16384, // snapshotPeriod
            0, // termSeconds
            100, // nominationExpiration
            100, // custodyPeriod
            200, // releasePeriod
            10, // maxNumOfValidators
            1, // minNumOfValidators
            100, // delegationThreshold
            100, // minDeposit
            500, // maxCandidateMetadataSize
            0 // era
        ];
        const newParams2 = [
            0x20, // maxExtraDataSize
            "tc", // networkID
            5, // minPayCost
            10, // minCustomCost
            4194304, // maxBodySize
            16384, // snapshotPeriod
            0, // termSeconds
            100, // nominationExpiration
            100, // custodyPeriod
            200, // releasePeriod
            10, // maxNumOfValidators
            1, // minNumOfValidators
            100, // delegationThreshold
            100, // minDeposit
            500, // maxCandidateMetadataSize
            0 // era
        ];
        const changeParams1: (number | string | (number | string)[])[] = [
            0xff,
            0,
            newParams1
        ];
        const changeParams2: (number | string | (number | string)[])[] = [
            0xff,
            1,
            newParams2
        ];
        const message1 = blake256(RLP.encode(changeParams1).toString("hex"));
        changeParams1.push(approvalEncoded(message1, aliceSecret));
        changeParams1.push(approvalEncoded(message1, bobSecret));
        changeParams1.push(approvalEncoded(message1, carolSecret));
        const message2 = blake256(RLP.encode(changeParams2).toString("hex"));
        changeParams2.push(approvalEncoded(message2, aliceSecret));
        changeParams2.push(approvalEncoded(message2, bobSecret));
        changeParams2.push(approvalEncoded(message2, carolSecret));

        {
            await node.rpc.devel!.stopSealing();
            const blockNumber = await node.rpc.chain.getBestBlockNumber();
            const seq = (await node.rpc.chain.getSeq({
                address: faucetAddress.toString(),
                blockNumber: null
            }))!;
            const tx1 = node.testFramework.core
                .createCustomTransaction({
                    handlerId: stakeActionHandlerId,
                    bytes: RLP.encode(changeParams1)
                })
                .sign({
                    secret: faucetSecret,
                    seq,
                    fee: 10
                });

            const trans1 = tx1.rlpBytes().toString("hex");
            const changeHash1 = await node.rpc.mempool.sendSignedTransaction({
                tx: `0x${trans1}`
            });
            const tx2 = node.testFramework.core
                .createCustomTransaction({
                    handlerId: stakeActionHandlerId,
                    bytes: RLP.encode(changeParams2)
                })
                .sign({
                    secret: faucetSecret,
                    seq: seq + 1,
                    fee: 10
                });
            const trans2 = tx2.rlpBytes().toString("hex");
            const changeHash2 = await node.rpc.mempool.sendSignedTransaction({
                tx: `0x${trans2}`
            });
            await node.rpc.devel!.startSealing();
            expect(
                await node.rpc.chain.containsTransaction({
                    transactionHash: changeHash1
                })
            ).be.true;
            expect(
                await node.rpc.chain.containsTransaction({
                    transactionHash: changeHash2
                })
            ).be.true;
            expect(await node.rpc.chain.getBestBlockNumber()).equal(
                blockNumber + 1
            );
        }

        const pay = await node.sendPayTx({ fee: 5 });
        expect(
            await node.rpc.chain.containsTransaction({
                transactionHash: "0x".concat(pay.hash().toString())
            })
        ).be.true;
        try {
            await node.sendPayTx({ fee: 4 });
            expect.fail();
        } catch (e) {
            expect(e).is.similarTo(ERROR.TOO_LOW_FEE);
        }
    });

    it("cannot change params with insufficient stakes", async function() {
        const newParams = [
            0x20, // maxExtraDataSize
            "tc", // networkID
            11, // minPayCost
            10, // minCustomCost
            4194304, // maxBodySize
            16384, // snapshotPeriod
            0, // termSeconds
            100, // nominationExpiration
            100, // custodyPeriod
            200, // releasePeriod
            10, // maxNumOfValidators
            1, // minNumOfValidators
            100, // delegationThreshold
            100, // minDeposit
            500, // maxCandidateMetadataSize
            0 // era
        ];
        const changeParams: (number | string | (number | string)[])[] = [
            0xff,
            0,
            newParams
        ];
        const message = blake256(RLP.encode(changeParams).toString("hex"));
        changeParams.push(approvalEncoded(message, aliceSecret));
        changeParams.push(approvalEncoded(message, carolSecret));

        {
            const tx = node.testFramework.core
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
                });
            const trans = tx.rlpBytes().toString("hex");
            const hash = await node.rpc.mempool.sendSignedTransaction({
                tx: `${trans}`
            });
            expect(
                await node.rpc.chain.containsTransaction({
                    transactionHash: hash
                })
            ).be.true;
        }

        {
            const tx2 = node.testFramework.core
                .createCustomTransaction({
                    handlerId: stakeActionHandlerId,
                    bytes: RLP.encode(changeParams)
                })
                .sign({
                    secret: faucetSecret,
                    seq:
                        (await node.rpc.chain.getSeq({
                            address: faucetAddress.toString(),
                            blockNumber: null
                        }))! + 1,
                    fee: 10
                });
            await node.sendSignedTransactionExpectedToFail(tx2, {
                error: "Invalid transaction seq Expected 1, found 0"
            });
        }
    });

    it("the amount of stakes not the number of stakeholders", async function() {
        const newParams = [
            0x20, // maxExtraDataSize
            "tc", // networkID
            11, // minPayCost
            10, // minCustomCost
            4194304, // maxBodySize
            16384, // snapshotPeriod
            0, // termSeconds
            100, // nominationExpiration
            100, // custodyPeriod
            200, // releasePeriod
            10, // maxNumOfValidators
            1, // minNumOfValidators
            100, // delegationThreshold
            100, // minDeposit
            500, // maxCandidateMetadataSize
            0 // era
        ];
        const changeParams: (number | string | (number | string)[])[] = [
            0xff,
            0,
            newParams
        ];
        const message = blake256(RLP.encode(changeParams).toString("hex"));
        changeParams.push(approvalEncoded(message, bobSecret));
        changeParams.push(approvalEncoded(message, carolSecret));

        const tx = node.testFramework.core
            .createCustomTransaction({
                handlerId: stakeActionHandlerId,
                bytes: RLP.encode(changeParams)
            })
            .sign({
                secret: faucetSecret,
                seq:
                    (await node.rpc.chain.getSeq({
                        address: faucetAddress.toString(),
                        blockNumber: null
                    }))! + 1,
                fee: 10
            });
        await node.sendSignedTransactionExpectedToFail(tx, {
            error: "Insufficient stakes:"
        });
    });

    it("needs more than half to change params", async function() {
        const newParams = [
            0x20, // maxExtraDataSize
            "tc", // networkID
            11, // minPayCost
            10, // minCustomCost
            4194304, // maxBodySize
            16384, // snapshotPeriod
            0, // termSeconds
            100, // nominationExpiration
            100, // custodyPeriod
            200, // releasePeriod
            10, // maxNumOfValidators
            1, // minNumOfValidators
            100, // delegationThreshold
            100, // minDeposit
            500, // maxCandidateMetadataSize
            0 // era
        ];

        const changeParams: (number | string | (number | string)[])[] = [
            0xff,
            0,
            newParams
        ];
        {
            const message = blake256(RLP.encode(changeParams).toString("hex"));
            changeParams.push(approvalEncoded(message, bobSecret));
            changeParams.push(approvalEncoded(message, carolSecret));

            const tx = node.testFramework.core
                .createCustomTransaction({
                    handlerId: stakeActionHandlerId,
                    bytes: RLP.encode(changeParams)
                })
                .sign({
                    secret: faucetSecret,
                    seq:
                        (await node.rpc.chain.getSeq({
                            address: faucetAddress.toString(),
                            blockNumber: null
                        }))! + 1,
                    fee: 10
                });
            await node.sendSignedTransactionExpectedToFail(tx, {
                error: "Insufficient"
            });
        }

        await sendStakeToken({
            node,
            senderAddress: aliceAddress,
            senderSecret: aliceSecret,
            receiverAddress: carolAddress,
            quantity: 1,
            fee: 1000
        });

        {
            const tx = node.testFramework.core
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
            ).not.be.null;
        }
        try {
            await node.sendPayTx({ fee: 10 });
            expect.fail();
        } catch (e) {
            expect(e).is.similarTo(ERROR.TOO_LOW_FEE);
        }
    });

    describe("with stake parameters", async function() {
        it("change", async function() {
            const newParams = [
                0x20, // maxExtraDataSize
                "tc", // networkID
                11, // minPayCost
                10, // minCustomCost
                4194304, // maxBodySize
                16384, // snapshotPeriod
                0, // termSeconds
                100, // nominationExpiration
                100, // custodyPeriod
                200, // releasePeriod
                10, // maxNumOfValidators
                1, // minNumOfValidators
                100, // delegationThreshold
                100, // minDeposit
                500, // maxCandidateMetadataSize
                0 // era
            ];
            const changeParams: (number | string | (number | string)[])[] = [
                0xff,
                0,
                newParams
            ];
            const message = blake256(RLP.encode(changeParams).toString("hex"));
            changeParams.push(approvalEncoded(message, aliceSecret));
            changeParams.push(approvalEncoded(message, carolSecret));

            {
                const tx = node.testFramework.core
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
            }

            try {
                await node.sendPayTx({ fee: 10 });
                expect.fail();
            } catch (e) {
                expect(e).is.similarTo(ERROR.TOO_LOW_FEE);
            }

            const params = await node.rpc.chain.getCommonParams({
                blockNumber: null
            });
            expect(+params!.minPayCost!).to.be.deep.equal(11);
        });

        it("nomination expiration cannot be zero", async function() {
            const newParams = [
                0x20, // maxExtraDataSize
                "tc", // networkID
                10, // minPayCost
                10, // minCustomCost
                4194304, // maxBodySize
                16384, // snapshotPeriod
                100, // termSeconds
                0, // nominationExpiration
                10, // custodyPeriod
                30, // releasePeriod
                101, // maxNumOfValidators
                100, // minNumOfValidators
                4, // delegationThreshold
                1000, // minDeposit
                128, // maxCandidateMetadataSize
                0 // era
            ];
            const changeParams: (number | string | (number | string)[])[] = [
                0xff,
                0,
                newParams
            ];
            const message = blake256(RLP.encode(changeParams).toString("hex"));
            changeParams.push(approvalEncoded(message, aliceSecret));
            changeParams.push(approvalEncoded(message, carolSecret));

            const tx = node.testFramework.core
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
                });
            const trans = tx.rlpBytes().toString("hex");
            try {
                await node.rpc.mempool.sendSignedTransaction({
                    tx: `0x${trans}`
                });
                expect.fail();
            } catch (e) {
                expect(e).is.similarTo(ERROR.ACTION_DATA_HANDLER_NOT_FOUND);
            }
        });

        it("custody period cannot be zero", async function() {
            const newParams = [
                0x20, // maxExtraDataSize
                "tc", // networkID
                10, // minPayCost
                10, // minCustomCost
                4194304, // maxBodySize
                16384, // snapshotPeriod
                100, // termSeconds
                10, // nominationExpiration
                0, // custodyPeriod
                30, // releasePeriod
                101, // maxNumOfValidators
                100, // minNumOfValidators
                4, // delegationThreshold
                1000, // minDeposit
                128, // maxCandidateMetadataSize
                0 // era
            ];
            const changeParams: (number | string | (number | string)[])[] = [
                0xff,
                0,
                newParams
            ];
            const message = blake256(RLP.encode(changeParams).toString("hex"));
            changeParams.push(approvalEncoded(message, aliceSecret));
            changeParams.push(approvalEncoded(message, carolSecret));

            const tx = node.testFramework.core
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
                });
            const trans = tx.rlpBytes().toString("hex");
            try {
                await node.rpc.mempool.sendSignedTransaction({
                    tx: `0x${trans}`
                });
                expect.fail();
            } catch (e) {
                expect(e).is.similarTo(ERROR.ACTION_DATA_HANDLER_NOT_FOUND);
            }
        });

        it("release period cannot be zero", async function() {
            const newParams = [
                0x20, // maxExtraDataSize
                "tc", // networkID
                10, // minPayCost
                10, // minCustomCost
                4194304, // maxBodySize
                16384, // snapshotPeriod
                100, // termSeconds
                10, // nominationExpiration
                10, // custodyPeriod
                0, // releasePeriod
                101, // maxNumOfValidators
                100, // minNumOfValidators
                4, // delegationThreshold
                1000, // minDeposit
                128, // maxCandidateMetadataSize
                0 // era
            ];
            const changeParams: (number | string | (number | string)[])[] = [
                0xff,
                0,
                newParams
            ];
            const message = blake256(RLP.encode(changeParams).toString("hex"));
            changeParams.push(approvalEncoded(message, aliceSecret));
            changeParams.push(approvalEncoded(message, carolSecret));

            const tx = node.testFramework.core
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
                });
            const trans = tx.rlpBytes().toString("hex");
            try {
                await node.rpc.mempool.sendSignedTransaction({
                    tx: `0x${trans}`
                });
                expect.fail();
            } catch (e) {
                expect(e).is.similarTo(ERROR.ACTION_DATA_HANDLER_NOT_FOUND);
            }
        });

        it("A release period cannot be equal to a custody period", async function() {
            const newParams = [
                0x20, // maxExtraDataSize
                "tc", // networkID
                10, // minPayCost
                10, // minCustomCost
                4194304, // maxBodySize
                16384, // snapshotPeriod
                100, // termSeconds
                10, // nominationExpiration
                10, // custodyPeriod
                10, // releasePeriod
                101, // maxNumOfValidators
                100, // minNumOfValidators
                4, // delegationThreshold
                1000, // minDeposit
                128, // maxCandidateMetadataSize
                0 // era
            ];
            const changeParams: (number | string | (number | string)[])[] = [
                0xff,
                0,
                newParams
            ];
            const message = blake256(RLP.encode(changeParams).toString("hex"));
            changeParams.push(approvalEncoded(message, aliceSecret));
            changeParams.push(approvalEncoded(message, carolSecret));

            const tx = node.testFramework.core
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
                });
            const trans = tx.rlpBytes().toString("hex");
            try {
                await node.rpc.mempool.sendSignedTransaction({
                    tx: `0x${trans}`
                });
                expect.fail();
            } catch (e) {
                expect(e).is.similarTo(ERROR.ACTION_DATA_HANDLER_NOT_FOUND);
            }
        });

        it("min deposit cannot be zero", async function() {
            const newParams = [
                0x20, // maxExtraDataSize
                "tc", // networkID
                10, // minPayCost
                10, // minCustomCost
                4194304, // maxBodySize
                16384, // snapshotPeriod
                100, // termSeconds
                10, // nominationExpiration
                10, // custodyPeriod
                20, // releasePeriod
                101, // maxNumOfValidators
                100, // minNumOfValidators
                4, // delegationThreshold
                0, // minDeposit
                128, // maxCandidateMetadataSize
                0 // era
            ];
            const changeParams: (number | string | (number | string)[])[] = [
                0xff,
                0,
                newParams
            ];
            const message = blake256(RLP.encode(changeParams).toString("hex"));
            changeParams.push(approvalEncoded(message, aliceSecret));
            changeParams.push(approvalEncoded(message, carolSecret));

            const tx = node.testFramework.core
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
                });
            const trans = tx.rlpBytes().toString("hex");
            try {
                await node.rpc.mempool.sendSignedTransaction({
                    tx: `0x${trans}`
                });
                expect.fail();
            } catch (e) {
                expect(e).is.similarTo(ERROR.ACTION_DATA_HANDLER_NOT_FOUND);
            }
        });

        it("delegation threshold cannot be zero", async function() {
            const newParams = [
                0x20, // maxExtraDataSize
                "tc", // networkID
                10, // minPayCost
                10, // minCustomCost
                4194304, // maxBodySize
                16384, // snapshotPeriod
                100, // termSeconds
                10, // nominationExpiration
                10, // custodyPeriod
                20, // releasePeriod
                101, // maxNumOfValidators
                100, // minNumOfValidators
                0, // delegationThreshold
                100, // minDeposit
                100, // maxCandidateMetadataSize
                0 // era
            ];
            const changeParams: (number | string | (number | string)[])[] = [
                0xff,
                0,
                newParams
            ];
            const message = blake256(RLP.encode(changeParams).toString("hex"));
            changeParams.push(approvalEncoded(message, aliceSecret));
            changeParams.push(approvalEncoded(message, carolSecret));

            const tx = node.testFramework.core
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
                });
            const trans = tx.rlpBytes().toString("hex");
            try {
                await node.rpc.mempool.sendSignedTransaction({
                    tx: `0x${trans}`
                });
                expect.fail();
            } catch (e) {
                expect(e).is.similarTo(ERROR.ACTION_DATA_HANDLER_NOT_FOUND);
            }
        });

        it("min number of validators cannot be zero", async function() {
            const newParams = [
                0x20, // maxExtraDataSize
                "tc", // networkID
                10, // minPayCost
                10, // minCustomCost
                4194304, // maxBodySize
                16384, // snapshotPeriod
                100, // termSeconds
                10, // nominationExpiration
                10, // custodyPeriod
                20, // releasePeriod
                101, // maxNumOfValidators
                0, // minNumOfValidators
                100, // delegationThreshold
                100, // minDeposit
                100, // maxCandidateMetadataSize
                0 // era
            ];
            const changeParams: (number | string | (number | string)[])[] = [
                0xff,
                0,
                newParams
            ];
            const message = blake256(RLP.encode(changeParams).toString("hex"));
            changeParams.push(approvalEncoded(message, aliceSecret));
            changeParams.push(approvalEncoded(message, carolSecret));

            const tx = node.testFramework.core
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
                });
            const trans = tx.rlpBytes().toString("hex");
            try {
                await node.rpc.mempool.sendSignedTransaction({
                    tx: `0x${trans}`
                });
                expect.fail();
            } catch (e) {
                expect(e).is.similarTo(ERROR.ACTION_DATA_HANDLER_NOT_FOUND);
            }
        });

        it("max number of validators cannot be zero", async function() {
            const newParams = [
                0x20, // maxExtraDataSize
                "tc", // networkID
                10, // minPayCost
                10, // minCustomCost
                4194304, // maxBodySize
                16384, // snapshotPeriod
                100, // termSeconds
                10, // nominationExpiration
                10, // custodyPeriod
                20, // releasePeriod
                0, // maxNumOfValidators
                10, // minNumOfValidators
                100, // delegationThreshold
                100, // minDeposit
                100, // maxCandidateMetadataSize
                0 // era
            ];
            const changeParams: (number | string | (number | string)[])[] = [
                0xff,
                0,
                newParams
            ];
            const message = blake256(RLP.encode(changeParams).toString("hex"));
            changeParams.push(approvalEncoded(message, aliceSecret));
            changeParams.push(approvalEncoded(message, carolSecret));

            const tx = node.testFramework.core
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
                });
            const trans = tx.rlpBytes().toString("hex");
            try {
                await node.rpc.mempool.sendSignedTransaction({
                    tx: `0x${trans}`
                });
                expect.fail();
            } catch (e) {
                expect(e).is.similarTo(ERROR.ACTION_DATA_HANDLER_NOT_FOUND);
            }
        });

        it("The maximum number of candidates cannot be equal to the minimum number of candidates", async function() {
            const newParams = [
                0x20, // maxExtraDataSize
                "tc", // networkID
                10, // minPayCost
                10, // minCustomCost
                4194304, // maxBodySize
                16384, // snapshotPeriod
                100, // termSeconds
                10, // nominationExpiration
                10, // custodyPeriod
                20, // releasePeriod
                99, // maxNumOfValidators
                100, // minNumOfValidators
                4, // delegationThreshold
                1000, // minDeposit
                128, // maxCandidateMetadataSize
                0 // era
            ];
            const changeParams: (number | string | (number | string)[])[] = [
                0xff,
                0,
                newParams
            ];
            const message = blake256(RLP.encode(changeParams).toString("hex"));
            changeParams.push(approvalEncoded(message, aliceSecret));
            changeParams.push(approvalEncoded(message, carolSecret));

            const tx = node.testFramework.core
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
                });
            const trans = tx.rlpBytes().toString("hex");
            try {
                await node.rpc.mempool.sendSignedTransaction({
                    tx: `0x${trans}`
                });
                expect.fail();
            } catch (e) {
                expect(e).is.similarTo(ERROR.ACTION_DATA_HANDLER_NOT_FOUND);
            }
        });
    });

    afterEach(async function() {
        if (this.currentTest!.state === "failed") {
            node.keepLogs();
        }
        await node.clean();
    });
});

async function sendStakeToken(params: {
    node: CodeChain;
    senderAddress: Address;
    senderSecret: string;
    receiverAddress: Address;
    quantity: number;
    fee?: number;
    seq?: number;
}): Promise<H256> {
    const {
        fee = 10,
        node,
        senderAddress,
        receiverAddress,
        senderSecret,
        quantity
    } = params;
    const {
        seq = (await node.rpc.chain.getSeq({
            address: senderAddress.toString()
        }))!
    } = params;
    const tx = node.testFramework.core
        .createCustomTransaction({
            handlerId: stakeActionHandlerId,
            bytes: Buffer.from(
                RLP.encode([
                    1,
                    receiverAddress.accountId.toEncodeObject(),
                    quantity
                ])
            )
        })
        .sign({
            secret: senderSecret,
            seq,
            fee
        });
    const trans = tx.rlpBytes().toString("hex");
    return new H256(
        await node.rpc.mempool.sendSignedTransaction({ tx: `0x${trans}` })
    );
}
