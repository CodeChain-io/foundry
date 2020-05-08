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
import { H256, Address } from "../primitives/src";
import "mocha";
import {
    aliceAddress,
    aliceSecret,
    bobAddress,
    carolAddress,
    daveAddress,
    faucetAddress,
    stakeActionHandlerId,
    validator0Address,
    validator0Secret,
    validator1Address,
    validator1Secret
} from "../helper/constants";
import { PromiseExpect } from "../helper/promise";
import CodeChain from "../helper/spawn";
import { toHex } from "../sdk/utils";
import * as RLP from "rlp";

describe("Staking", function() {
    const promiseExpect = new PromiseExpect();
    const chain = `${__dirname}/../scheme/solo.json`;
    let node: CodeChain;

    beforeEach(async function() {
        node = new CodeChain({
            chain,
            argv: ["--author", validator0Address.toString(), "--force-sealing"]
        });
        await node.start();
        await node.sendPayTx({ recipient: aliceAddress, quantity: 100_000 });
        await node.sendPayTx({
            recipient: validator0Address,
            quantity: 100_000
        });
        await node.sendPayTx({
            recipient: validator1Address,
            quantity: 100_000
        });
    });

    async function getAllStakingInfo() {
        const validatorAddresses = [
            faucetAddress,
            validator0Address,
            validator1Address,
            aliceAddress,
            bobAddress,
            carolAddress,
            daveAddress
        ];
        const stackholderaddress = `0x${toHex(
            RLP.encode(["StakeholderAddresses"])
        )}`;
        const amounts = await promiseExpect.shouldFulfill(
            "get customActionData",
            Promise.all(
                validatorAddresses.map(addr =>
                    node.rpc.engine.getCustomActionData({
                        handlerId: stakeActionHandlerId,
                        bytes: `0x${toHex(
                            RLP.encode([
                                "Account",
                                addr.pubkey.toEncodeObject()
                            ])
                        )}`
                    })
                )
            )
        );
        const stakeholders = await promiseExpect.shouldFulfill(
            "get customActionData",
            node.rpc.engine.getCustomActionData({
                handlerId: stakeActionHandlerId,
                bytes: stackholderaddress,
                blockNumber: null
            })
        );
        return { amounts, stakeholders };
    }

    async function getAllDelegation() {
        const validatorAddresses = [
            faucetAddress,
            validator0Address,
            validator1Address,
            aliceAddress,
            bobAddress,
            carolAddress,
            daveAddress
        ];
        const delegations = await promiseExpect.shouldFulfill(
            "get customActionData",
            Promise.all(
                validatorAddresses.map(addr =>
                    node.rpc.engine.getCustomActionData({
                        handlerId: stakeActionHandlerId,
                        bytes: `0x${toHex(
                            RLP.encode([
                                "Delegation",
                                addr.pubkey.toEncodeObject()
                            ])
                        )}`
                    })
                )
            )
        );
        return delegations;
    }

    async function sendStakeToken(params: {
        senderAddress: Address;
        senderSecret: string;
        receiverAddress: Address;
        quantity: number;
        fee?: number;
        seq?: number;
    }): Promise<H256> {
        const { fee = 10 } = params;
        const seq =
            params.seq == null
                ? (await node.rpc.chain.getSeq({
                      address: params.senderAddress.toString()
                  }))!
                : params.seq;

        const hashInString = await promiseExpect.shouldFulfill(
            "sendSignTransaction",
            node.rpc.mempool.sendSignedTransaction({
                tx: node.testFramework.core
                    .createTransferCCSTransaction({
                        recipient: params.receiverAddress,
                        quantity: params.quantity
                    })
                    .sign({
                        secret: params.senderSecret,
                        seq,
                        fee
                    })
                    .rlpBytes()
                    .toString("hex")
            })
        );
        return new H256(hashInString);
    }

    async function delegateToken(params: {
        senderAddress: Address;
        senderSecret: string;
        receiverAddress: Address;
        quantity: number;
        fee?: number;
        seq?: number;
    }): Promise<H256> {
        const { fee = 10 } = params;
        const seq =
            params.seq == null
                ? (await node.rpc.chain.getSeq({
                      address: params.senderAddress.toString()
                  }))!
                : params.seq;

        const hashInString = await promiseExpect.shouldFulfill(
            "sendSignTransaction",
            node.rpc.mempool.sendSignedTransaction({
                tx: node.testFramework.core
                    .createDelegateCCSTransaction({
                        delegatee: params.receiverAddress,
                        quantity: params.quantity
                    })
                    .sign({
                        secret: params.senderSecret,
                        seq,
                        fee
                    })
                    .rlpBytes()
                    .toString("hex")
            })
        );
        return new H256(hashInString);
    }

    async function revokeToken(params: {
        senderAddress: Address;
        senderSecret: string;
        delegateeAddress: Address;
        quantity: number;
        fee?: number;
        seq?: number;
    }): Promise<H256> {
        const { fee = 10 } = params;
        const seq =
            params.seq == null
                ? (await node.rpc.chain.getSeq({
                      address: params.senderAddress.toString()
                  }))!
                : params.seq;

        const hashInString = await promiseExpect.shouldFulfill(
            "sendSignTransaction",
            node.rpc.mempool.sendSignedTransaction({
                tx: node.testFramework.core
                    .createRevokeTransaction({
                        delegatee: params.delegateeAddress,
                        quantity: params.quantity
                    })
                    .sign({
                        secret: params.senderSecret,
                        seq,
                        fee
                    })
                    .rlpBytes()
                    .toString("hex")
            })
        );
        return new H256(hashInString);
    }

    async function selfNominate(params: {
        senderAddress: Address;
        senderSecret: string;
        deposit: number;
        metadata: Buffer | null;
        fee?: number;
        seq?: number;
    }): Promise<H256> {
        const { fee = 10, deposit, metadata } = params;
        const seq =
            params.seq == null
                ? (await node.rpc.chain.getSeq({
                      address: params.senderAddress.toString()
                  }))!
                : params.seq;

        const hashInString = await promiseExpect.shouldFulfill(
            "sendSignTransaction",
            node.rpc.mempool.sendSignedTransaction({
                tx: node.testFramework.core
                    .createSelfNominateTransaction({
                        deposit,
                        metadata: metadata || Buffer.from([])
                    })
                    .sign({
                        secret: params.senderSecret,
                        seq,
                        fee
                    })
                    .rlpBytes()
                    .toString("hex")
            })
        );
        return new H256(hashInString);
    }

    it("should have proper initial stake tokens", async function() {
        const { amounts, stakeholders } = await getAllStakingInfo();
        expect(amounts).to.be.deep.equal([
            null,
            null,
            null,
            toHex(RLP.encode(40000)),
            toHex(RLP.encode(30000)),
            toHex(RLP.encode(20000)),
            toHex(RLP.encode(10000))
        ]);
        expect(stakeholders).to.be.equal(
            toHex(
                RLP.encode(
                    [
                        aliceAddress.pubkey.toEncodeObject(),
                        bobAddress.pubkey.toEncodeObject(),
                        carolAddress.pubkey.toEncodeObject(),
                        daveAddress.pubkey.toEncodeObject()
                    ].sort()
                )
            )
        );
    });

    it("should send stake tokens", async function() {
        await sendStakeToken({
            senderAddress: aliceAddress,
            senderSecret: aliceSecret,
            receiverAddress: validator0Address,
            quantity: 100
        });

        const { amounts, stakeholders } = await getAllStakingInfo();
        expect(amounts).to.be.deep.equal([
            null,
            toHex(RLP.encode(100)),
            null,
            toHex(RLP.encode(40000 - 100)),
            toHex(RLP.encode(30000)),
            toHex(RLP.encode(20000)),
            toHex(RLP.encode(10000))
        ]);
        expect(stakeholders).to.be.equal(
            toHex(
                RLP.encode(
                    [
                        aliceAddress.pubkey.toEncodeObject(),
                        carolAddress.pubkey.toEncodeObject(),
                        validator0Address.pubkey.toEncodeObject(),
                        daveAddress.pubkey.toEncodeObject(),
                        bobAddress.pubkey.toEncodeObject()
                    ].sort()
                )
            )
        );
    }).timeout(60_000);

    it("doesn't leave zero balance account after transfer", async function() {
        await sendStakeToken({
            senderAddress: aliceAddress,
            senderSecret: aliceSecret,
            receiverAddress: validator0Address,
            quantity: 40000
        });

        const { amounts, stakeholders } = await getAllStakingInfo();
        expect(amounts).to.be.deep.equal([
            null,
            toHex(RLP.encode(40000)),
            null,
            null,
            toHex(RLP.encode(30000)),
            toHex(RLP.encode(20000)),
            toHex(RLP.encode(10000))
        ]);
        expect(stakeholders).to.be.equal(
            toHex(
                RLP.encode(
                    [
                        carolAddress.pubkey.toEncodeObject(),
                        validator0Address.pubkey.toEncodeObject(),
                        daveAddress.pubkey.toEncodeObject(),
                        bobAddress.pubkey.toEncodeObject()
                    ].sort()
                )
            )
        );
    }).timeout(60_000);

    it("can delegate tokens", async function() {
        await selfNominate({
            senderAddress: validator0Address,
            senderSecret: validator0Secret,
            deposit: 0,
            metadata: null
        });

        await delegateToken({
            senderAddress: aliceAddress,
            senderSecret: aliceSecret,
            receiverAddress: validator0Address,
            quantity: 100
        });

        const { amounts } = await getAllStakingInfo();
        expect(amounts).to.be.deep.equal([
            null,
            null,
            null,
            toHex(RLP.encode(40000 - 100)),
            toHex(RLP.encode(30000)),
            toHex(RLP.encode(20000)),
            toHex(RLP.encode(10000))
        ]);

        const delegations = await getAllDelegation();
        expect(delegations).to.be.deep.equal([
            null,
            null,
            null,
            toHex(
                RLP.encode([[validator0Address.pubkey.toEncodeObject(), 100]])
            ),
            null,
            null,
            null
        ]);
    });

    it("doesn't leave zero balanced account after delegate", async function() {
        await selfNominate({
            senderAddress: validator0Address,
            senderSecret: validator0Secret,
            deposit: 0,
            metadata: null
        });

        await delegateToken({
            senderAddress: aliceAddress,
            senderSecret: aliceSecret,
            receiverAddress: validator0Address,
            quantity: 40000
        });

        const { amounts } = await getAllStakingInfo();
        expect(amounts).to.be.deep.equal([
            null,
            null,
            null,
            null,
            toHex(RLP.encode(30000)),
            toHex(RLP.encode(20000)),
            toHex(RLP.encode(10000))
        ]);

        const delegations = await getAllDelegation();
        expect(delegations).to.be.deep.equal([
            null,
            null,
            null,
            toHex(
                RLP.encode([[validator0Address.pubkey.toEncodeObject(), 40000]])
            ),
            null,
            null,
            null
        ]);
    });

    it("can revoke tokens", async function() {
        await selfNominate({
            senderAddress: validator0Address,
            senderSecret: validator0Secret,
            deposit: 0,
            metadata: null
        });

        await delegateToken({
            senderAddress: aliceAddress,
            senderSecret: aliceSecret,
            receiverAddress: validator0Address,
            quantity: 100
        });

        await revokeToken({
            senderAddress: aliceAddress,
            senderSecret: aliceSecret,
            delegateeAddress: validator0Address,
            quantity: 50
        });

        const { amounts } = await getAllStakingInfo();
        expect(amounts).to.be.deep.equal([
            null,
            null,
            null,
            toHex(RLP.encode(40000 - 100 + 50)),
            toHex(RLP.encode(30000)),
            toHex(RLP.encode(20000)),
            toHex(RLP.encode(10000))
        ]);

        const delegations = await getAllDelegation();
        expect(delegations).to.be.deep.equal([
            null,
            null,
            null,
            toHex(
                RLP.encode([[validator0Address.pubkey.toEncodeObject(), 50]])
            ),
            null,
            null,
            null
        ]);
    });

    it("cannot revoke more than delegated", async function() {
        await selfNominate({
            senderAddress: validator0Address,
            senderSecret: validator0Secret,
            deposit: 0,
            metadata: null
        });

        await delegateToken({
            senderAddress: aliceAddress,
            senderSecret: aliceSecret,
            receiverAddress: validator0Address,
            quantity: 100
        });

        await node.rpc.devel!.stopSealing();
        await node.sendPayTx({
            recipient: faucetAddress,
            secret: validator0Secret,
            quantity: 1,
            seq: (await node.rpc.chain.getSeq({
                address: validator0Address.toString()
            }))!
        });
        const hash = await revokeToken({
            senderAddress: aliceAddress,
            senderSecret: aliceSecret,
            delegateeAddress: validator0Address,
            quantity: 200
        });
        await node.rpc.devel!.startSealing();

        expect(
            await node.rpc.mempool.getErrorHint({
                transactionHash: `0x${hash.toString()}`
            })
        ).not.to.be.null;

        const { amounts } = await getAllStakingInfo();
        expect(amounts).to.be.deep.equal([
            null,
            null,
            null,
            toHex(RLP.encode(40000 - 100)),
            toHex(RLP.encode(30000)),
            toHex(RLP.encode(20000)),
            toHex(RLP.encode(10000))
        ]);

        const delegations = await getAllDelegation();
        expect(delegations).to.be.deep.equal([
            null,
            null,
            null,
            toHex(
                RLP.encode([[validator0Address.pubkey.toEncodeObject(), 100]])
            ),
            null,
            null,
            null
        ]);
    });

    it("revoking all should clear delegation", async function() {
        await selfNominate({
            senderAddress: validator0Address,
            senderSecret: validator0Secret,
            deposit: 0,
            metadata: null
        });

        await delegateToken({
            senderAddress: aliceAddress,
            senderSecret: aliceSecret,
            receiverAddress: validator0Address,
            quantity: 100
        });

        await revokeToken({
            senderAddress: aliceAddress,
            senderSecret: aliceSecret,
            delegateeAddress: validator0Address,
            quantity: 100
        });

        const { amounts } = await getAllStakingInfo();
        expect(amounts).to.be.deep.equal([
            null,
            null,
            null,
            toHex(RLP.encode(40000)),
            toHex(RLP.encode(30000)),
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
        // alice: 40000, bob: 30000, carol: 20000, dave: 10000
        const fee = 1000;
        await sendStakeToken({
            senderAddress: aliceAddress,
            senderSecret: aliceSecret,
            receiverAddress: validator0Address,
            quantity: 20000,
            fee
        });
        // alice: 20000, bob: 30000, carol: 20000, dave: 10000, val0: 20000,

        const blockNumber = await node.getBestBlockNumber();

        const oldAliceBalance = +(await node.rpc.chain.getBalance({
            address: aliceAddress.toString(),
            blockNumber: blockNumber - 1
        }))!;
        // FIXME: Change Number to U64
        const aliceBalance = +(await node.rpc.chain.getBalance({
            address: aliceAddress.toString()
        }))!;
        expect(aliceBalance).to.be.deep.equal(oldAliceBalance - fee);

        const oldBobBalance = +(await node.rpc.chain.getBalance({
            address: bobAddress.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const bobBalance = +(await node.rpc.chain.getBalance({
            address: bobAddress.toString()
        }))!;
        expect(bobBalance).to.be.deep.equal(oldBobBalance);

        const oldCarolBalance = +(await node.rpc.chain.getBalance({
            address: carolAddress.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const carolBalance = +(await node.rpc.chain.getBalance({
            address: carolAddress.toString()
        }))!;
        expect(carolBalance).to.be.deep.equal(oldCarolBalance);

        const oldDaveBalance = +(await node.rpc.chain.getBalance({
            address: daveAddress.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const daveBalance = +(await node.rpc.chain.getBalance({
            address: daveAddress.toString()
        }))!;
        expect(daveBalance).to.be.deep.equal(oldDaveBalance);

        const oldValidator0Balance = +(await node.rpc.chain.getBalance({
            address: validator0Address.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const validator0Balance = +(await node.rpc.chain.getBalance({
            address: validator0Address.toString()
        }))!;
        expect(validator0Balance).to.be.deep.equal(oldValidator0Balance);
    });

    it("get fee even if it delegated stakes to other", async function() {
        await selfNominate({
            senderAddress: validator1Address,
            senderSecret: validator1Secret,
            deposit: 0,
            metadata: null
        });

        // alice: 40000, bob: 30000, carol 20000, dave: 10000
        await sendStakeToken({
            senderAddress: aliceAddress,
            senderSecret: aliceSecret,
            receiverAddress: validator0Address,
            quantity: 20000,
            fee: 1000
        });

        const fee = 100;
        await node.sendPayTx({
            recipient: validator0Address,
            quantity: fee
        });

        // alice: 20000, bob: 30000, carol 20000, dave: 10000, val0: 20000
        await delegateToken({
            senderAddress: validator0Address,
            senderSecret: validator0Secret,
            receiverAddress: validator1Address,
            quantity: 20000,
            fee
        });
        // alice: 20000, bob: 30000, carol 20000, dave: 10000, val0: 0 (delegated 20000 to val1), val1: 0

        const delegations = await getAllDelegation();
        expect(delegations).to.be.deep.equal([
            null,
            toHex(
                RLP.encode([[validator1Address.pubkey.toEncodeObject(), 20000]])
            ),
            null,
            null,
            null,
            null,
            null
        ]);

        const blockNumber = await node.getBestBlockNumber();

        const oldAliceBalance = +(await node.rpc.chain.getBalance({
            address: aliceAddress.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const aliceBalance = +(await node.rpc.chain.getBalance({
            address: aliceAddress.toString()
        }))!;
        expect(aliceBalance).to.be.deep.equal(oldAliceBalance);

        const oldBobBalance = +(await node.rpc.chain.getBalance({
            address: bobAddress.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const bobBalance = +(await node.rpc.chain.getBalance({
            address: bobAddress.toString()
        }))!;
        expect(bobBalance).to.be.deep.equal(oldBobBalance);

        const oldCarolBalance = +(await node.rpc.chain.getBalance({
            address: carolAddress.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const carolBalance = +(await node.rpc.chain.getBalance({
            address: carolAddress.toString()
        }))!;
        expect(carolBalance).to.be.deep.equal(oldCarolBalance);

        const oldDaveBalance = +(await node.rpc.chain.getBalance({
            address: daveAddress.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const daveBalance = +(await node.rpc.chain.getBalance({
            address: daveAddress.toString()
        }))!;
        expect(daveBalance).to.be.deep.equal(oldDaveBalance);

        const oldValidator0Balance = +(await node.rpc.chain.getBalance({
            address: validator0Address.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const validator0Balance = +(await node.rpc.chain.getBalance({
            address: validator0Address.toString()
        }))!;
        expect(validator0Balance).to.be.deep.equal(oldValidator0Balance - fee);

        const oldValidator1Balance = +(await node.rpc.chain.getBalance({
            address: validator1Address.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const validator1Balance = +(await node.rpc.chain.getBalance({
            address: validator1Address.toString()
        }))!;
        expect(validator1Balance).to.be.deep.equal(oldValidator1Balance);
    });

    it("get fee even if it delegated stakes to other stakeholder", async function() {
        await selfNominate({
            senderAddress: validator1Address,
            senderSecret: validator1Secret,
            deposit: 0,
            metadata: null
        });

        // alice: 40000, bob: 30000, carol 20000, dave: 10000
        await sendStakeToken({
            senderAddress: aliceAddress,
            senderSecret: aliceSecret,
            receiverAddress: validator0Address,
            quantity: 20000,
            fee: 1000
        });

        // alice: 20000, bob: 30000, carol 20000, dave: 10000, val0 20000
        await sendStakeToken({
            senderAddress: aliceAddress,
            senderSecret: aliceSecret,
            receiverAddress: validator1Address,
            quantity: 10000,
            fee: 1000
        });
        // alice: 10000, bob: 30000, carol 20000, dave: 10000, val0 20000, val1: 10000

        const fee = 567;
        await node.sendPayTx({
            recipient: validator0Address,
            quantity: fee,
            fee
        });

        await delegateToken({
            senderAddress: validator0Address,
            senderSecret: validator0Secret,
            receiverAddress: validator1Address,
            quantity: 20000,
            fee
        });
        // alice: 10000, bob: 30000, carol 20000, dave: 10000, val0 20000 (delegated 20000 to val1), val1: 10000

        const delegations = await getAllDelegation();
        expect(delegations).to.be.deep.equal([
            null,
            toHex(
                RLP.encode([[validator1Address.pubkey.toEncodeObject(), 20000]])
            ),
            null,
            null,
            null,
            null,
            null
        ]);

        const blockNumber = await node.getBestBlockNumber();

        const oldAliceBalance = +(await node.rpc.chain.getBalance({
            address: aliceAddress.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const aliceBalance = +(await node.rpc.chain.getBalance({
            address: aliceAddress.toString()
        }))!;
        expect(aliceBalance).to.be.deep.equal(oldAliceBalance);

        const oldBobBalance = +(await node.rpc.chain.getBalance({
            address: bobAddress.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const bobBalance = +(await node.rpc.chain.getBalance({
            address: bobAddress.toString()
        }))!;
        expect(bobBalance).to.be.deep.equal(oldBobBalance);

        const oldFaucetBalance = +(await node.rpc.chain.getBalance({
            address: faucetAddress.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const faucetBalance = +(await node.rpc.chain.getBalance({
            address: faucetAddress.toString()
        }))!;
        expect(faucetBalance).to.be.deep.equal(oldFaucetBalance);

        const oldValidator0Balance = +(await node.rpc.chain.getBalance({
            address: validator0Address.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const validator0Balance = +(await node.rpc.chain.getBalance({
            address: validator0Address.toString()
        }))!;
        expect(validator0Balance).to.be.deep.equal(oldValidator0Balance - fee);

        const oldValidator1Balance = +(await node.rpc.chain.getBalance({
            address: validator1Address.toString(),
            blockNumber: blockNumber - 1
        }))!;
        const validator1Balance = +(await node.rpc.chain.getBalance({
            address: validator1Address.toString()
        }))!;
        expect(validator1Balance).to.be.deep.equal(oldValidator1Balance);
    });

    afterEach(async function() {
        if (this.currentTest!.state === "failed") {
            node.keepLogs();
        }
        await node.clean();
        promiseExpect.checkFulfilled();
    });
});
