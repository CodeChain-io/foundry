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
import { PlatformAddress } from "codechain-primitives/lib";
import { toHex } from "codechain-sdk/lib/utils";
import "mocha";
import {
    aliceAddress,
    bobAddress,
    faucetAddress,
    faucetSecret,
    stakeActionHandlerId,
    validator0Address,
    validator0Secret,
    validator0BlsPublic,
    validator0PopSignature,
    validator1Address,
    validator1Secret,
    validator1BlsPublic,
    validator1PopSignature,
    validator2Address,
    validator3Address
} from "../helper/constants";
import { PromiseExpect, wait } from "../helper/promise";
import CodeChain from "../helper/spawn";

const RLP = require("rlp");

describe("Staking", function() {
    this.timeout(60_000);
    const promiseExpect = new PromiseExpect();
    let nodes: CodeChain[];

    beforeEach(async function() {
        this.timeout(60_000);

        const validatorAddresses = [
            validator0Address,
            validator1Address,
            validator2Address,
            validator3Address
        ];
        nodes = validatorAddresses.map(address => {
            return new CodeChain({
                chain: `${__dirname}/../scheme/tendermint-int.json`,
                argv: [
                    "--engine-signer",
                    address.toString(),
                    "--password-path",
                    "test/tendermint/password.json",
                    "--force-sealing",
                    "--no-discovery"
                ],
                additionalKeysPath: "tendermint/keys"
            });
        });
        await Promise.all(nodes.map(node => node.start()));
        await prepare();
    });

    async function prepare() {
        await promiseExpect.shouldFulfill(
            "connect",
            Promise.all([
                nodes[0].connect(nodes[1]),
                nodes[0].connect(nodes[2]),
                nodes[0].connect(nodes[3]),
                nodes[1].connect(nodes[2]),
                nodes[1].connect(nodes[3]),
                nodes[2].connect(nodes[3])
            ])
        );
        await promiseExpect.shouldFulfill(
            "wait peers",
            Promise.all([
                nodes[0].waitPeers(4 - 1),
                nodes[1].waitPeers(4 - 1),
                nodes[2].waitPeers(4 - 1),
                nodes[3].waitPeers(4 - 1)
            ])
        );

        // give some ccc to pay fee
        const pay1 = await nodes[0].sendPayTx({
            recipient: validator0Address,
            quantity: 100000,
            fee: 12,
            seq: 0
        });
        const pay2 = await nodes[0].sendPayTx({
            recipient: validator1Address,
            quantity: 100000,
            fee: 12,
            seq: 1
        });
        while (
            !(await nodes[0].rpc.chain.containsTransaction({
                transactionHash: `0x${pay1.hash()}`
            })) ||
            !(await nodes[0].rpc.chain.containsTransaction({
                transactionHash: `0x${pay2.hash()}`
            }))
        ) {
            await wait(500);
        }
    }

    async function getAllStakingInfo() {
        const validatorAddresses = [
            faucetAddress,
            validator0Address,
            validator1Address,
            validator2Address,
            validator3Address,
            aliceAddress,
            bobAddress
        ];
        const amounts = await promiseExpect.shouldFulfill(
            "get customActionData",
            Promise.all(
                validatorAddresses.map(addr =>
                    nodes[0].rpc.engine.getCustomActionData({
                        handlerId: stakeActionHandlerId,
                        bytes: `0x${toHex(
                            RLP.encode([
                                "Account",
                                addr.accountId.toEncodeObject()
                            ])
                        )}`
                    })
                )
            )
        );
        const stakeholders = await promiseExpect.shouldFulfill(
            "get customActionData",
            nodes[0].rpc.engine.getCustomActionData({
                handlerId: stakeActionHandlerId,
                bytes: `0x${toHex(RLP.encode(["StakeholderAddresses"]))}`
            })
        );
        return { amounts, stakeholders };
    }

    async function getAllDelegation() {
        const validatorAddresses = [
            faucetAddress,
            validator0Address,
            validator1Address,
            validator2Address,
            validator3Address,
            aliceAddress,
            bobAddress
        ];
        const delegations = await promiseExpect.shouldFulfill(
            "get customActionData",
            Promise.all(
                validatorAddresses.map(addr =>
                    nodes[0].rpc.engine.getCustomActionData({
                        handlerId: stakeActionHandlerId,
                        bytes: `0x${toHex(
                            RLP.encode([
                                "Delegation",
                                addr.accountId.toEncodeObject()
                            ])
                        )}`
                    })
                )
            )
        );
        return delegations;
    }

    async function sendStakeToken(params: {
        senderAddress: PlatformAddress;
        senderSecret: string;
        receiverAddress: PlatformAddress;
        quantity: number;
        fee?: number;
        seq?: number;
    }): Promise<string> {
        const { fee = 10 } = params;
        const seq =
            params.seq == null
                ? (await nodes[0].rpc.chain.getSeq({
                      address: params.senderAddress.toString()
                  }))!
                : params.seq;

        const tx = nodes[0].sdk.core
            .createCustomTransaction({
                handlerId: stakeActionHandlerId,
                bytes: Buffer.from(
                    RLP.encode([
                        1,
                        params.receiverAddress.accountId.toEncodeObject(),
                        params.quantity
                    ])
                )
            })
            .sign({
                secret: params.senderSecret,
                seq,
                fee
            });
        const trans = tx.rlpBytes().toString("hex");
        return promiseExpect.shouldFulfill(
            "sendSignTransaction",
            nodes[0].rpc.mempool.sendSignedTransaction({ tx: `${trans}` })
        );
    }

    async function delegateToken(params: {
        senderAddress: PlatformAddress;
        senderSecret: string;
        receiverAddress: PlatformAddress;
        quantity: number;
        fee?: number;
        seq?: number;
    }): Promise<string> {
        const { fee = 10 } = params;
        const seq =
            params.seq == null
                ? (await nodes[0].rpc.chain.getSeq({
                      address: params.senderAddress.toString()
                  }))!
                : params.seq;

        const tx = nodes[0].sdk.core
            .createCustomTransaction({
                handlerId: stakeActionHandlerId,
                bytes: Buffer.from(
                    RLP.encode([
                        2,
                        params.receiverAddress.accountId.toEncodeObject(),
                        params.quantity
                    ])
                )
            })
            .sign({
                secret: params.senderSecret,
                seq,
                fee
            });
        const trans = tx.rlpBytes().toString("hex");
        return promiseExpect.shouldFulfill(
            "sendSignTransaction",
            nodes[0].rpc.mempool.sendSignedTransaction({ tx: `${trans}` })
        );
    }

    async function revokeToken(params: {
        senderAddress: PlatformAddress;
        senderSecret: string;
        delegateeAddress: PlatformAddress;
        quantity: number;
        fee?: number;
        seq?: number;
    }): Promise<string> {
        const { fee = 10 } = params;
        const seq =
            params.seq == null
                ? (await nodes[0].rpc.chain.getSeq({
                      address: params.senderAddress.toString()
                  }))!
                : params.seq;

        const tx = nodes[0].sdk.core
            .createCustomTransaction({
                handlerId: stakeActionHandlerId,
                bytes: Buffer.from(
                    RLP.encode([
                        3,
                        params.delegateeAddress.accountId.toEncodeObject(),
                        params.quantity
                    ])
                )
            })
            .sign({
                secret: params.senderSecret,
                seq,
                fee
            });
        const trans = tx.rlpBytes().toString("hex");
        return promiseExpect.shouldFulfill(
            "sendSignTransaction",
            nodes[0].rpc.mempool.sendSignedTransaction({ tx: `0x${trans}` })
        );
    }

    async function selfNominate(params: {
        senderAddress: PlatformAddress;
        senderSecret: string;
        senderBlsPublic: string;
        senderPopSignature: string;
        deposit: number;
        metadata: Buffer | null;
        fee?: number;
        seq?: number;
        waitForEnd?: boolean;
    }): Promise<string> {
        const {
            fee = 10,
            deposit,
            senderBlsPublic,
            senderPopSignature,
            metadata,
            waitForEnd = true
        } = params;
        const seq =
            params.seq == null
                ? (await nodes[0].rpc.chain.getSeq({
                      address: params.senderAddress.toString()
                  }))!
                : params.seq;

        const tx = nodes[0].sdk.core
            .createCustomTransaction({
                handlerId: stakeActionHandlerId,
                bytes: Buffer.from(
                    RLP.encode([
                        4,
                        deposit,
                        senderBlsPublic,
                        senderPopSignature,
                        metadata
                    ])
                )
            })
            .sign({
                secret: params.senderSecret,
                seq,
                fee
            });
        const trans = tx.rlpBytes().toString("hex");
        const promise = promiseExpect.shouldFulfill(
            "sendSignTransaction",
            nodes[0].rpc.mempool.sendSignedTransaction({ tx: `${trans}` })
        );
        if (waitForEnd) {
            const hash = await promise;
            while (
                !(await nodes[0].rpc.chain.containsTransaction({
                    transactionHash: `${hash}`
                }))
            ) {
                await wait(500);
            }
        }
        return promise;
    }

    it("should have proper initial stake tokens", async function() {
        const { amounts, stakeholders } = await getAllStakingInfo();
        expect(amounts).to.be.deep.equal([
            toHex(RLP.encode(70000)),
            null,
            null,
            null,
            null,
            toHex(RLP.encode(20000)),
            toHex(RLP.encode(10000))
        ]);

        expect(stakeholders).to.be.equal(
            toHex(
                RLP.encode([
                    aliceAddress.accountId.toEncodeObject(),
                    faucetAddress.accountId.toEncodeObject(),
                    bobAddress.accountId.toEncodeObject()
                ])
            )
        );
    });

    it("should send stake tokens", async function() {
        const hash = await sendStakeToken({
            senderAddress: faucetAddress,
            senderSecret: faucetSecret,
            receiverAddress: validator0Address,
            quantity: 100
        });
        while (
            !(await nodes[0].rpc.chain.containsTransaction({
                transactionHash: `${hash}`
            }))
        ) {
            await wait(500);
        }

        const { amounts, stakeholders } = await getAllStakingInfo();
        expect(amounts).to.be.deep.equal([
            toHex(RLP.encode(70000 - 100)),
            toHex(RLP.encode(100)),
            null,
            null,
            null,
            toHex(RLP.encode(20000)),
            toHex(RLP.encode(10000))
        ]);
        expect(stakeholders).to.be.equal(
            toHex(
                RLP.encode(
                    [
                        faucetAddress.accountId.toEncodeObject(),
                        aliceAddress.accountId.toEncodeObject(),
                        validator0Address.accountId.toEncodeObject(),
                        bobAddress.accountId.toEncodeObject()
                    ].sort()
                )
            )
        );
    }).timeout(60_000);

    it("doesn't leave zero balance account after transfer", async function() {
        const hash = await sendStakeToken({
            senderAddress: faucetAddress,
            senderSecret: faucetSecret,
            receiverAddress: validator0Address,
            quantity: 70000
        });
        while (
            !(await nodes[0].rpc.chain.containsTransaction({
                transactionHash: `${hash}`
            }))
        ) {
            await wait(500);
        }

        const { amounts, stakeholders } = await getAllStakingInfo();
        expect(amounts).to.be.deep.equal([
            null,
            toHex(RLP.encode(70000)),
            null,
            null,
            null,
            toHex(RLP.encode(20000)),
            toHex(RLP.encode(10000))
        ]);
        expect(stakeholders).to.be.equal(
            toHex(
                RLP.encode(
                    [
                        aliceAddress.accountId.toEncodeObject(),
                        validator0Address.accountId.toEncodeObject(),
                        bobAddress.accountId.toEncodeObject()
                    ].sort()
                )
            )
        );
    }).timeout(60_000);

    it("can delegate tokens", async function() {
        await selfNominate({
            senderAddress: validator0Address,
            senderSecret: validator0Secret,
            senderBlsPublic: validator0BlsPublic,
            senderPopSignature: validator0PopSignature,
            deposit: 0,
            metadata: null
        });

        const hash = await delegateToken({
            senderAddress: faucetAddress,
            senderSecret: faucetSecret,
            receiverAddress: validator0Address,
            quantity: 100
        });
        while (
            !(await nodes[0].rpc.chain.containsTransaction({
                transactionHash: `${hash}`
            }))
        ) {
            await wait(500);
        }

        const { amounts } = await getAllStakingInfo();
        expect(amounts).to.be.deep.equal([
            toHex(RLP.encode(70000 - 100)),
            null,
            null,
            null,
            null,
            toHex(RLP.encode(20000)),
            toHex(RLP.encode(10000))
        ]);

        const delegations = await getAllDelegation();
        expect(delegations).to.be.deep.equal([
            toHex(
                RLP.encode([
                    [validator0Address.accountId.toEncodeObject(), 100]
                ])
            ),
            null,
            null,
            null,
            null,
            null,
            null
        ]);
    });

    it("doesn't leave zero balanced account after delegate", async function() {
        await selfNominate({
            senderAddress: validator0Address,
            senderSecret: validator0Secret,
            senderBlsPublic: validator0BlsPublic,
            senderPopSignature: validator0PopSignature,
            deposit: 0,
            metadata: null
        });

        const hash = await delegateToken({
            senderAddress: faucetAddress,
            senderSecret: faucetSecret,
            receiverAddress: validator0Address,
            quantity: 70000
        });
        while (
            !(await nodes[0].rpc.chain.containsTransaction({
                transactionHash: `${hash}`
            }))
        ) {
            await wait(500);
        }

        const { amounts } = await getAllStakingInfo();
        expect(amounts).to.be.deep.equal([
            null,
            null,
            null,
            null,
            null,
            toHex(RLP.encode(20000)),
            toHex(RLP.encode(10000))
        ]);

        const delegations = await getAllDelegation();
        expect(delegations).to.be.deep.equal([
            toHex(
                RLP.encode([
                    [validator0Address.accountId.toEncodeObject(), 70000]
                ])
            ),
            null,
            null,
            null,
            null,
            null,
            null
        ]);
    });

    it("cannot delegate to non-candidate", async function() {
        // give some ccs to delegate.

        const hash1 = await sendStakeToken({
            senderAddress: faucetAddress,
            senderSecret: faucetSecret,
            receiverAddress: validator0Address,
            quantity: 200
        });
        while (
            !(await nodes[0].rpc.chain.containsTransaction({
                transactionHash: `${hash1}`
            }))
        ) {
            await wait(500);
        }

        const blockNumber = await nodes[0].getBestBlockNumber();
        const seq = (await nodes[0].rpc.chain.getSeq({
            address: validator0Address.toString()
        }))!;
        const pay = await nodes[0].sendPayTx({
            recipient: faucetAddress,
            secret: validator0Secret,
            quantity: 1,
            seq
        });

        // delegate
        const hash = await delegateToken({
            senderAddress: validator0Address,
            senderSecret: validator0Secret,
            receiverAddress: faucetAddress,
            quantity: 100,
            seq: seq + 1
        });
        await nodes[0].waitBlockNumber(blockNumber + 1);

        while (
            !(await nodes[0].rpc.chain.containsTransaction({
                transactionHash: `0x${pay.hash().toString()}`
            }))
        ) {
            await wait(500);
        }
        const err0 = await nodes[0].rpc.mempool.getErrorHint({
            transactionHash: `${hash}`
        });
        const err1 = await nodes[1].rpc.mempool.getErrorHint({
            transactionHash: `${hash}`
        });
        const err2 = await nodes[2].rpc.mempool.getErrorHint({
            transactionHash: `${hash}`
        });
        const err3 = await nodes[3].rpc.mempool.getErrorHint({
            transactionHash: `${hash}`
        });
        expect(err0 || err1 || err2 || err3).not.null;
    });

    it("can revoke tokens", async function() {
        await selfNominate({
            senderAddress: validator0Address,
            senderSecret: validator0Secret,
            senderBlsPublic: validator0BlsPublic,
            senderPopSignature: validator0PopSignature,
            deposit: 0,
            metadata: null
        });

        const delegateHash = await delegateToken({
            senderAddress: faucetAddress,
            senderSecret: faucetSecret,
            receiverAddress: validator0Address,
            quantity: 100
        });
        while (
            !(await nodes[0].rpc.chain.containsTransaction({
                transactionHash: `${delegateHash}`
            }))
        ) {
            await wait(500);
        }

        const hash = await revokeToken({
            senderAddress: faucetAddress,
            senderSecret: faucetSecret,
            delegateeAddress: validator0Address,
            quantity: 50
        });

        while (
            !(await nodes[0].rpc.chain.containsTransaction({
                transactionHash: `${hash}`
            }))
        ) {
            await wait(500);
        }

        const { amounts } = await getAllStakingInfo();
        expect(amounts).to.be.deep.equal([
            toHex(RLP.encode(70000 - 100 + 50)),
            null,
            null,
            null,
            null,
            toHex(RLP.encode(20000)),
            toHex(RLP.encode(10000))
        ]);

        const delegations = await getAllDelegation();
        expect(delegations).to.be.deep.equal([
            toHex(
                RLP.encode([[validator0Address.accountId.toEncodeObject(), 50]])
            ),
            null,
            null,
            null,
            null,
            null,
            null
        ]);
    });

    it("cannot revoke more than delegated", async function() {
        await selfNominate({
            senderAddress: validator0Address,
            senderSecret: validator0Secret,
            senderBlsPublic: validator0BlsPublic,
            senderPopSignature: validator0PopSignature,
            deposit: 0,
            metadata: null
        });

        const delegateHash = await delegateToken({
            senderAddress: faucetAddress,
            senderSecret: faucetSecret,
            receiverAddress: validator0Address,
            quantity: 100
        });
        while (
            !(await nodes[0].rpc.chain.containsTransaction({
                transactionHash: `${delegateHash}`
            }))
        ) {
            await wait(500);
        }

        const blockNumber = await nodes[0].getBestBlockNumber();
        const seq = (await nodes[0].rpc.chain.getSeq({
            address: faucetAddress.toString()
        }))!;
        const pay = await nodes[0].sendPayTx({
            recipient: faucetAddress,
            secret: faucetSecret,
            quantity: 1,
            seq
        });

        // revoke
        const hash = await revokeToken({
            senderAddress: faucetAddress,
            senderSecret: faucetSecret,
            delegateeAddress: validator0Address,
            quantity: 200,
            seq: seq + 1
        });
        await nodes[0].waitBlockNumber(blockNumber + 1);

        while (
            !(await nodes[0].rpc.chain.containsTransaction({
                transactionHash: `0x${pay.hash().toString()}`
            }))
        ) {
            await wait(500);
        }
        const err0 = await nodes[0].rpc.mempool.getErrorHint({
            transactionHash: `${hash}`
        });
        const err1 = await nodes[1].rpc.mempool.getErrorHint({
            transactionHash: `${hash}`
        });
        const err2 = await nodes[2].rpc.mempool.getErrorHint({
            transactionHash: `${hash}`
        });
        const err3 = await nodes[3].rpc.mempool.getErrorHint({
            transactionHash: `${hash}`
        });
        expect(err0 || err1 || err2 || err3).not.null;

        const { amounts } = await getAllStakingInfo();
        expect(amounts).to.be.deep.equal([
            toHex(RLP.encode(70000 - 100)),
            null,
            null,
            null,
            null,
            toHex(RLP.encode(20000)),
            toHex(RLP.encode(10000))
        ]);

        const delegations = await getAllDelegation();
        expect(delegations).to.be.deep.equal([
            toHex(
                RLP.encode([
                    [validator0Address.accountId.toEncodeObject(), 100]
                ])
            ),
            null,
            null,
            null,
            null,
            null,
            null
        ]);
    });

    it("revoking all should clear delegation", async function() {
        await selfNominate({
            senderAddress: validator0Address,
            senderSecret: validator0Secret,
            senderBlsPublic: validator0BlsPublic,
            senderPopSignature: validator0PopSignature,
            deposit: 0,
            metadata: null
        });

        const delegateHash = await delegateToken({
            senderAddress: faucetAddress,
            senderSecret: faucetSecret,
            receiverAddress: validator0Address,
            quantity: 100
        });
        while (
            !(await nodes[0].rpc.chain.containsTransaction({
                transactionHash: `${delegateHash}`
            }))
        ) {
            await wait(500);
        }

        const hash = await revokeToken({
            senderAddress: faucetAddress,
            senderSecret: faucetSecret,
            delegateeAddress: validator0Address,
            quantity: 100
        });

        while (
            !(await nodes[0].rpc.chain.containsTransaction({
                transactionHash: `${hash}`
            }))
        ) {
            await wait(500);
        }

        const { amounts } = await getAllStakingInfo();
        expect(amounts).to.be.deep.equal([
            toHex(RLP.encode(70000)),
            null,
            null,
            null,
            null,
            toHex(RLP.encode(20000)),
            toHex(RLP.encode(10000))
        ]);

        const delegations = await getAllDelegation();
        expect(delegations).to.be.deep.equal([
            null,
            null,
            null,
            null,
            null,
            null,
            null
        ]);
    });

    it("get fee in proportion to holding stakes", async function() {
        // faucet: 70000, alice: 20000, bob: 10000
        const fee = 1000;
        const hash = await sendStakeToken({
            senderAddress: faucetAddress,
            senderSecret: faucetSecret,
            receiverAddress: validator0Address,
            quantity: 50000,
            fee
        });
        while (
            !(await nodes[0].rpc.chain.containsTransaction({
                transactionHash: `${hash}`
            }))
        ) {
            await wait(500);
        }
        // faucet: 20000, alice: 20000, bob: 10000, val0: 50000,

        const blockNumber = await nodes[0].getBestBlockNumber();
        const minCustomCost = require("../scheme/tendermint-int.json").params
            .minCustomCost;

        const oldAliceBalance = +(await nodes[0].rpc.chain.getBalance({
            address: aliceAddress.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const aliceBalance = +(await nodes[0].rpc.chain.getBalance({
            address: aliceAddress.toString()
        }))!;
        expect(aliceBalance).to.be.deep.equal(
            oldAliceBalance + Math.floor((minCustomCost * 2) / 10)
        );

        const oldBobBalance = +(await nodes[0].rpc.chain.getBalance({
            address: bobAddress.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const bobBalance = +(await nodes[0].rpc.chain.getBalance({
            address: bobAddress.toString()
        }))!;
        expect(bobBalance).to.be.deep.equal(
            oldBobBalance + Math.floor((minCustomCost * 1) / 10)
        );

        const oldFaucetBalance = +(await nodes[0].rpc.chain.getBalance({
            address: faucetAddress.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const faucetBalance = +(await nodes[0].rpc.chain.getBalance({
            address: faucetAddress.toString()
        }))!;
        expect(faucetBalance).to.be.deep.equal(
            oldFaucetBalance + (Math.floor((minCustomCost * 2) / 10) - fee)
        );

        const author = (await nodes[0].rpc.chain.getBlockByNumber({
            blockNumber
        }))!.author;
        const oldValidator0Balance = +(await nodes[0].rpc.chain.getBalance({
            address: validator0Address.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const validator0Balance = +(await nodes[0].rpc.chain.getBalance({
            address: validator0Address.toString()
        }))!;
        if (author === validator0Address.value) {
            expect(validator0Balance).to.be.deep.equal(
                oldValidator0Balance +
                    (Math.floor((minCustomCost * 5) / 10) + fee - minCustomCost)
            );
        } else {
            expect(validator0Balance).to.be.deep.equal(
                oldValidator0Balance + Math.floor((minCustomCost * 5) / 10)
            );
            const oldAuthorBalance = +(await nodes[0].rpc.chain.getBalance({
                address: author.toString(),
                blockNumber: blockNumber - 1
            }))!;
            const authorBalance = (await nodes[0].rpc.chain.getBalance({
                address: author.toString()
            }))!;
            expect(authorBalance).to.be.deep.equal(
                oldAuthorBalance + (fee - minCustomCost)
            );
        }
    });

    it("get fee even if it delegated stakes to other", async function() {
        await selfNominate({
            senderAddress: validator1Address,
            senderSecret: validator1Secret,
            senderBlsPublic: validator1BlsPublic,
            senderPopSignature: validator1PopSignature,
            deposit: 0,
            metadata: null
        });

        // faucet: 70000, alice: 20000, bob: 10000
        const hash1 = await sendStakeToken({
            senderAddress: faucetAddress,
            senderSecret: faucetSecret,
            receiverAddress: validator0Address,
            quantity: 50000,
            fee: 1000
        });
        while (
            !(await nodes[0].rpc.chain.containsTransaction({
                transactionHash: `${hash1}`
            }))
        ) {
            await wait(500);
        }

        const fee = 100;
        const payHash = (
            await nodes[0].sendPayTx({
                recipient: validator0Address,
                quantity: fee
            })
        ).hash();

        while (
            !(await nodes[0].rpc.chain.containsTransaction({
                transactionHash: `0x${payHash.toString()}`
            }))
        ) {
            await wait(500);
        }

        // faucet: 20000, alice: 20000, bob: 10000, val0: 50000
        const hash2 = await delegateToken({
            senderAddress: validator0Address,
            senderSecret: validator0Secret,
            receiverAddress: validator1Address,
            quantity: 50000,
            fee
        });

        while (
            !(await nodes[0].rpc.chain.containsTransaction({
                transactionHash: `${hash2}`
            }))
        ) {
            await wait(500);
        }
        // faucet: 20000, alice: 20000, bob: 10000, val0: 0 (delegated 50000 to val1), val1: 0

        const blockNumber = await nodes[0].getBestBlockNumber();
        const minCustomCost = require("../scheme/tendermint-int.json").params
            .minCustomCost;

        const oldAliceBalance = +(await nodes[0].rpc.chain.getBalance({
            address: aliceAddress.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const aliceBalance = +(await nodes[0].rpc.chain.getBalance({
            address: aliceAddress.toString()
        }))!;
        expect(aliceBalance).to.be.deep.equal(
            oldAliceBalance + Math.floor((minCustomCost * 2) / 10)
        );

        const oldBobBalance = +(await nodes[0].rpc.chain.getBalance({
            address: bobAddress.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const bobBalance = +(await nodes[0].rpc.chain.getBalance({
            address: bobAddress.toString()
        }))!;
        expect(bobBalance).to.be.deep.equal(
            oldBobBalance + Math.floor((minCustomCost * 1) / 10)
        );

        const oldFaucetBalance = +(await nodes[0].rpc.chain.getBalance({
            address: faucetAddress.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const faucetBalance = +(await nodes[0].rpc.chain.getBalance({
            address: faucetAddress.toString()
        }))!;
        expect(faucetBalance).to.be.deep.equal(
            oldFaucetBalance + Math.floor((minCustomCost * 2) / 10)
        );

        const author = (await nodes[0].rpc.chain.getBlockByNumber({
            blockNumber
        }))!.author;
        const oldValidator0Balance = +(await nodes[0].rpc.chain.getBalance({
            address: validator0Address.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const validator0Balance = +(await nodes[0].rpc.chain.getBalance({
            address: validator0Address.toString()
        }))!;
        if (author === validator0Address.value) {
            expect(validator0Balance).to.be.deep.equal(
                oldValidator0Balance +
                    Math.floor((minCustomCost * 5) / 10) -
                    fee +
                    fee -
                    minCustomCost
            );
        } else {
            expect(validator0Balance).to.be.deep.equal(
                oldValidator0Balance +
                    (Math.floor((minCustomCost * 5) / 10) - fee)
            );

            const oldValidator1Balance = +(await nodes[0].rpc.chain.getBalance({
                address: validator1Address.toString(),
                blockNumber: blockNumber - 1
            }))!;
            const validator1Balance = +(await nodes[0].rpc.chain.getBalance({
                address: validator1Address.toString()
            }))!;
            if (author === validator1Address.value) {
                expect(validator1Balance).to.be.deep.equal(
                    oldValidator1Balance + fee - minCustomCost
                );
            } else {
                expect(validator1Balance.toString(10)).to.be.deep.equal(
                    oldValidator1Balance.toString(10)
                );

                const oldAuthorBalance = +(await nodes[0].rpc.chain.getBalance({
                    address: author.toString(),
                    blockNumber: blockNumber - 1
                }))!;
                const authorBalance = +(await nodes[0].rpc.chain.getBalance({
                    address: author.toString()
                }))!;
                expect(authorBalance).to.be.deep.equal(
                    oldAuthorBalance + fee - minCustomCost
                );
            }
        }
    });

    it("get fee even if it delegated stakes to other stakeholder", async function() {
        await selfNominate({
            senderAddress: validator1Address,
            senderSecret: validator1Secret,
            senderBlsPublic: validator1BlsPublic,
            senderPopSignature: validator1PopSignature,
            deposit: 0,
            metadata: null
        });

        // faucet: 70000, alice: 20000, bob: 10000
        const hash1 = await sendStakeToken({
            senderAddress: faucetAddress,
            senderSecret: faucetSecret,
            receiverAddress: validator0Address,
            quantity: 30000,
            fee: 1000
        });
        while (
            !(await nodes[0].rpc.chain.containsTransaction({
                transactionHash: `${hash1}`
            }))
        ) {
            await wait(500);
        }

        // faucet: 40000, alice: 20000, bob: 10000, val0: 30000
        const hash2 = await sendStakeToken({
            senderAddress: faucetAddress,
            senderSecret: faucetSecret,
            receiverAddress: validator1Address,
            quantity: 30000,
            fee: 1000
        });
        while (
            !(await nodes[0].rpc.chain.containsTransaction({
                transactionHash: `${hash2}`
            }))
        ) {
            await wait(500);
        }

        const fee = 567;
        const payHash = (
            await nodes[0].sendPayTx({
                recipient: validator0Address,
                quantity: fee,
                fee
            })
        ).hash();
        while (
            !(await nodes[0].rpc.chain.containsTransaction({
                transactionHash: `0x${payHash}`
            }))
        ) {
            await wait(500);
        }

        // faucet: 10000, alice: 20000, bob: 10000, val0: 30000, val1: 30000
        const hash3 = await delegateToken({
            senderAddress: validator0Address,
            senderSecret: validator0Secret,
            receiverAddress: validator1Address,
            quantity: 30000,
            fee
        });

        while (
            !(await nodes[0].rpc.chain.containsTransaction({
                transactionHash: `${hash3}`
            }))
        ) {
            await wait(500);
        }
        // faucet: 20000, alice: 20000, bob: 10000, val0: 0 (delegated 30000 to val1), val1: 30000

        const blockNumber = await nodes[0].getBestBlockNumber();
        const minCustomCost = require("../scheme/tendermint-int.json").params
            .minCustomCost;

        const oldAliceBalance = +(await nodes[0].rpc.chain.getBalance({
            address: aliceAddress.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const aliceBalance = +(await nodes[0].rpc.chain.getBalance({
            address: aliceAddress.toString()
        }))!;

        expect(aliceBalance).to.equal(
            oldAliceBalance + Math.floor((minCustomCost * 2) / 10)
        );

        const oldBobBalance = +(await nodes[0].rpc.chain.getBalance({
            address: bobAddress.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const bobBalance = +(await nodes[0].rpc.chain.getBalance({
            address: bobAddress.toString()
        }))!;
        expect(bobBalance).to.be.deep.equal(
            oldBobBalance + Math.floor((minCustomCost * 1) / 10)
        );
        const oldFaucetBalance = +(await nodes[0].rpc.chain.getBalance({
            address: faucetAddress.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const faucetBalance = +(await nodes[0].rpc.chain.getBalance({
            address: faucetAddress.toString()
        }))!;
        expect(faucetBalance).to.be.deep.equal(
            oldFaucetBalance + Math.floor((minCustomCost * 1) / 10)
        );
        const author = (await nodes[0].rpc.chain.getBlockByNumber({
            blockNumber
        }))!.author;
        const oldValidator0Balance = +(await nodes[0].rpc.chain.getBalance({
            address: validator0Address.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const validator0Balance = +(await nodes[0].rpc.chain.getBalance({
            address: validator0Address.toString()
        }))!;
        if (author === validator0Address.value) {
            expect(validator0Balance).to.be.deep.equal(
                oldValidator0Balance +
                    Math.floor((minCustomCost * 3) / 10) -
                    fee +
                    fee -
                    minCustomCost
            );
        } else {
            expect(validator0Balance).to.be.deep.equal(
                oldValidator0Balance +
                    (Math.floor((minCustomCost * 3) / 10) - fee)
            );

            const oldValidator1Balance = +(await nodes[0].rpc.chain.getBalance({
                address: validator1Address.toString(),
                blockNumber: blockNumber - 1
            }))!;
            const validator1Balance = +(await nodes[0].rpc.chain.getBalance({
                address: validator1Address.toString()
            }))!;
            if (author === validator1Address.value) {
                expect(validator1Balance).to.be.deep.equal(
                    oldValidator1Balance +
                        Math.floor((minCustomCost * 3) / 10) +
                        fee -
                        minCustomCost
                );
            } else {
                expect(validator1Balance).to.be.deep.equal(
                    oldValidator1Balance + Math.floor((minCustomCost * 3) / 10)
                );

                const oldAuthorBalance = +(await nodes[0].rpc.chain.getBalance({
                    address: author.toString(),
                    blockNumber: blockNumber - 1
                }))!;
                const authorBalance = +(await nodes[0].rpc.chain.getBalance({
                    address: author.toString()
                }))!;
                expect(authorBalance).to.be.deep.equal(
                    oldAuthorBalance + fee - minCustomCost
                );
            }
        }
    });

    afterEach(async function() {
        if (this.currentTest!.state === "failed") {
            nodes.map(node => node.keepLogs());
        }
        await Promise.all(nodes.map(node => node.clean()));
        promiseExpect.checkFulfilled();
    });
});
