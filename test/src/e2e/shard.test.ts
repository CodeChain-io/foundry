// Copyright 2019 Kodebox, Inc.
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
import * as chaiAsPromised from "chai-as-promised";
chai.use(chaiAsPromised);
import "mocha";
import {
    aliceAddress,
    aliceSecret,
    bobAddress,
    bobSecret,
    faucetAddress,
    faucetSecret
} from "../helper/constants";
import CodeChain from "../helper/spawn";

const expect = chai.expect;

describe.skip("CreateShard", function() {
    let node: CodeChain;
    before(async function() {
        node = new CodeChain({ argv: ["--allow-create-shard"] });
        await node.start();
    });

    it("Create 1 shard", async function() {
        const seq: number = (await node.rpc.chain.getSeq({
            address: faucetAddress.toString(),
            blockNumber: null
        }))!;

        await node.rpc.mempool.sendSignedTransaction({
            tx: node.testFramework.core
                .createPayTransaction({ recipient: aliceAddress, quantity: 1 })
                .sign({ secret: faucetSecret, seq, fee: 10 })
                .rlpBytes()
                .toString("hex")
        });

        const tx = node.testFramework.core
            .createCreateShardTransaction({ users: [aliceAddress] })
            .sign({ secret: faucetSecret, seq: seq + 1, fee: 10 });
        const beforeBlockNumber = await node.rpc.chain.getBestBlockNumber();
        expect(
            node.rpc.chain.getShardIdByHash({
                transactionHash: `0x${tx.hash().toString()}`,
                blockNumber: null
            })
        ).to.be.null;
        await node.rpc.mempool.sendSignedTransaction({
            tx: tx.rlpBytes().toString("hex")
        });
        expect(
            await node.rpc.chain.containsTransaction({
                transactionHash: `0x${tx.hash().toString()}`
            })
        ).be.true;
        expect(
            await node.rpc.chain.getTransaction({
                transactionHash: `0x${tx.hash().toString()}`
            })
        ).not.null;
        const afterShardId = (await node.rpc.chain.getShardIdByHash({
            transactionHash: `0x${tx.hash().toString()}`,
            blockNumber: null
        }))!;
        expect(afterShardId).not.to.be.null;

        const shardOwners = await node.rpc.chain.getShardOwners({
            shardId: afterShardId,
            blockNumber: null
        });
        expect(shardOwners).to.deep.equal([faucetAddress.value]); // The creator becomes the owner.
        const shardUsers = await node.rpc.chain.getShardUsers({
            shardId: afterShardId,
            blockNumber: null
        });
        expect(shardUsers).to.deep.equal([aliceAddress.value]);

        expect(
            await node.rpc.chain.getShardOwners({
                shardId: afterShardId,
                blockNumber: beforeBlockNumber
            })
        ).to.be.null;
        expect(
            await node.rpc.chain.getShardUsers({
                shardId: afterShardId,
                blockNumber: beforeBlockNumber
            })
        ).to.be.null;
    });

    it("setShardUsers", async function() {
        const seq: number = (await node.rpc.chain.getSeq({
            address: faucetAddress.toString(),
            blockNumber: null
        }))!;
        await node.rpc.mempool.sendSignedTransaction({
            tx: node.testFramework.core
                .createPayTransaction({ recipient: aliceAddress, quantity: 1 })
                .sign({ secret: faucetSecret, seq, fee: 10 })
                .rlpBytes()
                .toString("hex")
        });
        await node.rpc.mempool.sendSignedTransaction({
            tx: node.testFramework.core
                .createPayTransaction({ recipient: bobAddress, quantity: 1 })
                .sign({ secret: faucetSecret, seq: seq + 1, fee: 10 })
                .rlpBytes()
                .toString("hex")
        });

        const tx = node.testFramework.core
            .createCreateShardTransaction({ users: [aliceAddress] })
            .sign({ secret: faucetSecret, seq: seq + 2, fee: 10 });
        await node.rpc.mempool.sendSignedTransaction({
            tx: tx.rlpBytes().toString("hex")
        });
        const shardId = (await node.rpc.chain.getShardIdByHash({
            transactionHash: `0x${tx.hash().toString()}`,
            blockNumber: null
        }))!;
        const users = [aliceAddress, bobAddress];
        const setShardUsers = node.testFramework.core
            .createSetShardUsersTransaction({ shardId, users })
            .sign({ secret: faucetSecret, seq: seq + 3, fee: 10 });
        await node.rpc.mempool.sendSignedTransaction({
            tx: setShardUsers.rlpBytes().toString("hex")
        });
        const shardUsers = (await node.rpc.chain.getShardIdByHash({
            transactionHash: `0x${tx.hash().toString()}`,
            blockNumber: null
        }))!;
        expect(shardUsers).to.deep.equal(users.map(user => user.value));
    });

    it("setShardOwners", async function() {
        const seq: number = (await node.rpc.chain.getSeq({
            address: faucetAddress.toString(),
            blockNumber: null
        }))!;
        await node.rpc.mempool.sendSignedTransaction({
            tx: node.testFramework.core
                .createPayTransaction({ recipient: aliceAddress, quantity: 1 })
                .sign({ secret: faucetSecret, seq, fee: 10 })
                .rlpBytes()
                .toString("hex")
        });
        await node.rpc.mempool.sendSignedTransaction({
            tx: node.testFramework.core
                .createPayTransaction({ recipient: bobAddress, quantity: 1 })
                .sign({ secret: faucetSecret, seq: seq + 1, fee: 10 })
                .rlpBytes()
                .toString("hex")
        });
        const tx = node.testFramework.core
            .createCreateShardTransaction({ users: [aliceAddress, bobAddress] })
            .sign({ secret: faucetSecret, seq: seq + 2, fee: 10 });
        await node.rpc.mempool.sendSignedTransaction({
            tx: tx.rlpBytes().toString("hex")
        });

        const shardId = (await node.rpc.chain.getShardIdByHash({
            transactionHash: `0x${tx.hash().toString()}`,
            blockNumber: null
        }))!;
        const owners = [aliceAddress, faucetAddress, bobAddress];
        const setShardOwners = node.testFramework.core
            .createSetShardOwnersTransaction({ shardId, owners })
            .sign({ secret: faucetSecret, seq: seq + 3, fee: 10 });
        await node.rpc.mempool.sendSignedTransaction({
            tx: setShardOwners.rlpBytes().toString("hex")
        });
        const shardOwners = await node.rpc.chain.getShardOwners({ shardId })!;
        expect(shardOwners).to.deep.equal(owners.map(owner => owner.value));
    });

    it("Create 2 shards", async function() {
        const seq: number = (await node.rpc.chain.getSeq({
            address: faucetAddress.toString(),
            blockNumber: null
        }))!;
        await node.rpc.mempool.sendSignedTransaction({
            tx: node.testFramework.core
                .createPayTransaction({ recipient: aliceAddress, quantity: 1 })
                .sign({ secret: faucetSecret, seq, fee: 10 })
                .rlpBytes()
                .toString("hex")
        });
        await node.rpc.mempool.sendSignedTransaction({
            tx: node.testFramework.core
                .createPayTransaction({ recipient: bobAddress, quantity: 1 })
                .sign({ secret: faucetSecret, seq: seq + 1, fee: 10 })
                .rlpBytes()
                .toString("hex")
        });
        const tx1 = node.testFramework.core
            .createCreateShardTransaction({ users: [aliceAddress, bobAddress] })
            .sign({ secret: faucetSecret, seq: seq + 2, fee: 10 });
        expect(
            await node.rpc.chain.getShardIdByHash({
                transactionHash: `0x${tx1.hash().toString()}`,
                blockNumber: null
            })
        ).to.be.null;
        await node.rpc.mempool.sendSignedTransaction({
            tx: tx1.rlpBytes().toString("hex")
        });
        expect(
            await node.rpc.chain.containsTransaction({
                transactionHash: `0x${tx1.hash().toString()}`
            })
        ).be.true;
        expect(
            await node.rpc.chain.getTransaction({
                transactionHash: `0x${tx1.hash().toString()}`
            })
        ).not.null;
        expect(
            await node.rpc.chain.getShardIdByHash({
                transactionHash: `0x${tx1.hash().toString()}`,
                blockNumber: null
            })
        ).not.to.be.null;

        const tx2 = node.testFramework.core
            .createCreateShardTransaction({ users: [aliceAddress, bobAddress] })
            .sign({ secret: faucetSecret, seq: seq + 3, fee: 10 });
        expect(
            await node.rpc.chain.getShardIdByHash({
                transactionHash: `0x${tx2.hash().toString()}`,
                blockNumber: null
            })
        ).to.be.null;
        await node.rpc.mempool.sendSignedTransaction({
            tx: tx2.rlpBytes().toString("hex")
        });
        expect(
            await node.rpc.chain.containsTransaction({
                transactionHash: `0x${tx2.hash().toString()}`
            })
        ).be.true;
        expect(
            await node.rpc.chain.getTransaction({
                transactionHash: `0x${tx2.hash().toString()}`
            })
        ).not.null;
        expect(
            await node.rpc.chain.getShardIdByHash({
                transactionHash: `0x${tx2.hash().toString()}`,
                blockNumber: null
            })
        ).not.to.be.null;
    });

    it("non-user cannot mint", async function() {
        const faucetSeq: number = (await node.rpc.chain.getSeq({
            address: faucetAddress.toString(),
            blockNumber: null
        }))!;
        await node.rpc.mempool.sendSignedTransaction({
            tx: node.testFramework.core
                .createPayTransaction({ recipient: aliceAddress, quantity: 1 })
                .sign({ secret: faucetSecret, seq: faucetSeq, fee: 10 })
                .rlpBytes()
                .toString("hex")
        });
        await node.rpc.mempool.sendSignedTransaction({
            tx: node.testFramework.core
                .createPayTransaction({ recipient: bobAddress, quantity: 1 })
                .sign({ secret: faucetSecret, seq: faucetSeq + 1, fee: 10 })
                .rlpBytes()
                .toString("hex")
        });
        const createShard = node.testFramework.core
            .createCreateShardTransaction({ users: [aliceAddress] })
            .sign({ secret: faucetSecret, seq: faucetSeq + 2, fee: 10 });
        await node.rpc.mempool.sendSignedTransaction({
            tx: createShard.rlpBytes().toString("hex")
        });
        const shardId = (await node.rpc.chain.getShardIdByHash({
            transactionHash: `0x${createShard.hash().toString()}`,
            blockNumber: null
        }))!;
        await node.rpc.mempool.sendSignedTransaction({
            tx: node.testFramework.core
                .createPayTransaction({
                    recipient: bobAddress,
                    quantity: 100
                })
                .sign({ secret: faucetSecret, seq: faucetSeq + 3, fee: 10 })
                .rlpBytes()
                .toString("hex")
        });

        const bobSeq: number = (await node.rpc.chain.getSeq({
            address: bobAddress.toString(),
            blockNumber: null
        }))!;
        const pay = node.testFramework.core
            .createPayTransaction({
                recipient: aliceAddress,
                quantity: 1
            })
            .sign({ secret: bobSecret, seq: bobSeq, fee: 10 });
        const mint = node.testFramework.core
            .createMintAssetTransaction({
                scheme: {
                    shardId,
                    metadata: "",
                    supply: "0xa"
                },
                recipient: await node.createP2PKHAddress()
            })
            .sign({ secret: bobSecret, seq: bobSeq + 1, fee: 10 });

        await node.rpc.devel!.stopSealing();
        const blockNumber = await node.rpc.chain.getBestBlockNumber();
        await node.rpc.mempool.sendSignedTransaction({
            tx: pay.rlpBytes().toString("hex")
        });
        await node.rpc.mempool.sendSignedTransaction({
            tx: mint.rlpBytes().toString("hex")
        });
        await node.rpc.devel!.startSealing();
        await node.waitBlockNumber(blockNumber + 1);

        const hint = await node.rpc.mempool.getErrorHint({
            transactionHash: mint.hash().toString()
        });
        expect(hint).includes("permission");
    });

    it("user can mint", async function() {
        const faucetSeq: number = (await node.rpc.chain.getSeq({
            address: faucetAddress.toString(),
            blockNumber: null
        }))!;
        const createShard = node.testFramework.core
            .createCreateShardTransaction({ users: [aliceAddress] })
            .sign({ secret: faucetSecret, seq: faucetSeq, fee: 10 });
        await node.rpc.mempool.sendSignedTransaction({
            tx: createShard.rlpBytes().toString("hex")
        });
        const shardId = (await node.rpc.chain.getShardIdByHash({
            transactionHash: `0x${createShard.hash().toString()}`,
            blockNumber: null
        }))!;
        await node.rpc.mempool.sendSignedTransaction({
            tx: node.testFramework.core
                .createPayTransaction({
                    recipient: aliceAddress,
                    quantity: 100
                })
                .sign({ secret: faucetSecret, seq: faucetSeq + 1, fee: 10 })
                .rlpBytes()
                .toString("hex")
        });

        const aliceSeq: number = (await node.rpc.chain.getSeq({
            address: aliceAddress.toString(),
            blockNumber: null
        }))!;
        const mint = node.testFramework.core
            .createMintAssetTransaction({
                scheme: {
                    shardId,
                    metadata: "",
                    supply: "0xa"
                },
                recipient: await node.createP2PKHAddress()
            })
            .sign({ secret: aliceSecret, seq: aliceSeq, fee: 10 });
        await node.rpc.mempool.sendSignedTransaction({
            tx: mint.rlpBytes().toString("hex")
        });

        expect(
            await node.rpc.chain.containsTransaction({
                transactionHash: `0x${mint.hash().toString()}`
            })
        ).be.true;
        expect(
            await node.rpc.chain.getTransaction({
                transactionHash: `0x${mint.hash().toString()}`
            })
        ).not.null;
        const hint = await node.rpc.mempool.getErrorHint({
            transactionHash: mint.hash().toString()
        });
        expect(hint).to.be.null;
    });

    it("non-user can mint after becoming a user", async function() {
        const faucetSeq: number = (await node.rpc.chain.getSeq({
            address: faucetAddress.toString(),
            blockNumber: null
        }))!;
        const createShard = node.testFramework.core
            .createCreateShardTransaction({ users: [aliceAddress] })
            .sign({ secret: faucetSecret, seq: faucetSeq, fee: 10 });
        await node.rpc.mempool.sendSignedTransaction({
            tx: createShard.rlpBytes().toString("hex")
        });
        const shardId = (await node.rpc.chain.getShardIdByHash({
            transactionHash: `0x${createShard.hash().toString()}`,
            blockNumber: null
        }))!;
        await node.rpc.mempool.sendSignedTransaction({
            tx: node.testFramework.core
                .createPayTransaction({
                    recipient: bobAddress,
                    quantity: 100
                })
                .sign({ secret: faucetSecret, seq: faucetSeq + 1, fee: 10 })
                .rlpBytes()
                .toString("hex")
        });

        const bobSeq: number = (await node.rpc.chain.getSeq({
            address: bobAddress.toString(),
            blockNumber: null
        }))!;
        const recipient = await node.createP2PKHAddress();
        const mint1 = node.testFramework.core.createMintAssetTransaction({
            scheme: {
                shardId,
                metadata: "",
                supply: "0xa"
            },
            recipient
        });
        const signedMint1 = mint1.sign({
            secret: bobSecret,
            seq: bobSeq + 1,
            fee: 30
        });

        const blockNumber = await node.rpc.chain.getBestBlockNumber();
        await node.rpc.devel!.stopSealing();
        await node.rpc.mempool.sendSignedTransaction({
            tx: node.testFramework.core
                .createPayTransaction({
                    recipient: aliceAddress,
                    quantity: 1
                })
                .sign({
                    secret: bobSecret,
                    seq: bobSeq,
                    fee: 10
                })
                .rlpBytes()
                .toString("hex")
        });
        await node.rpc.mempool.sendSignedTransaction({
            tx: signedMint1.rlpBytes().toString("hex")
        });
        await node.rpc.devel!.startSealing();
        await node.waitBlockNumber(blockNumber + 1);
        expect(
            await node.rpc.mempool.getTransactionResultsByTracker({
                tracker: mint1.tracker().toString()
            })
        ).deep.equal([false]);
        const hint = await node.rpc.mempool.getErrorHint({
            transactionHash: signedMint1.hash().toString()
        });
        expect(hint).includes("permission");

        const newUsers = [aliceAddress, bobAddress];
        const setShardUsers = node.testFramework.core
            .createSetShardUsersTransaction({ shardId, users: newUsers })
            .sign({ secret: faucetSecret, seq: faucetSeq + 2, fee: 10 });
        await node.rpc.mempool.sendSignedTransaction({
            tx: setShardUsers.rlpBytes().toString("hex")
        });
        const shardUsers = (await node.rpc.chain.getShardUsers({ shardId }))!;
        expect(shardUsers).to.deep.equal(newUsers.map(user => user.value));

        const mint2 = node.testFramework.core.createMintAssetTransaction({
            scheme: {
                shardId,
                metadata: "",
                supply: "0xa"
            },
            recipient
        });
        expect(mint1.tracker().value).equal(mint2.tracker().value);
        const signedMint2 = mint2.sign({
            secret: bobSecret,
            seq: bobSeq + 1,
            fee: 20
        });
        await node.rpc.mempool.sendSignedTransaction({
            tx: signedMint2.rlpBytes().toString("hex")
        });

        expect(
            await node.rpc.chain.containsTransaction({
                transactionHash: `0x${signedMint2.hash().toString()}`
            })
        ).be.true;
        expect(
            await node.rpc.chain.getTransaction({
                transactionHash: `0x${signedMint2.hash().toString()}`
            })
        ).not.null;
        expect(
            await node.rpc.mempool.getTransactionResultsByTracker({
                tracker: mint2.tracker().toString()
            })
        ).deep.equal([false, true]);
        expect(
            await node.rpc.mempool.getErrorHint({
                transactionHash: signedMint2.hash().toString()
            })
        ).to.be.null;
    });

    after(async function() {
        await node.clean();
    });
});

describe.skip("Cannot create shard without allow-create-shard flag", function() {
    let node: CodeChain;
    before(async function() {
        node = new CodeChain();
        await node.start();
    });

    it("Create 1 shard", async function() {
        const seq: number = (await node.rpc.chain.getSeq({
            address: faucetAddress.toString(),
            blockNumber: null
        }))!;

        await node.rpc.mempool.sendSignedTransaction({
            tx: node.testFramework.core
                .createPayTransaction({ recipient: aliceAddress, quantity: 1 })
                .sign({ secret: faucetSecret, seq, fee: 10 })
                .rlpBytes()
                .toString("hex")
        });

        const tx = node.testFramework.core
            .createCreateShardTransaction({ users: [aliceAddress] })
            .sign({ secret: faucetSecret, seq: seq + 1, fee: 10 });
        expect(
            await node.rpc.chain.getShardIdByHash({
                transactionHash: `0x${tx.hash().toString()}`,
                blockNumber: null
            })
        ).be.null;
        expect(
            node.rpc.mempool.sendSignedTransaction({
                tx: tx.rlpBytes().toString("hex")
            })
        ).be.rejected;
        expect(
            await node.rpc.chain.containsTransaction({
                transactionHash: `0x${tx.hash().toString()}`
            })
        ).be.false;
        expect(
            await node.rpc.chain.getTransaction({
                transactionHash: `0x${tx.hash().toString()}`
            })
        ).be.null;
        const afterShardId = await node.rpc.chain.getShardIdByHash({
            transactionHash: `0x${tx.hash().toString()}`,
            blockNumber: null
        })!;
        expect(afterShardId).be.null;
    });

    after(async function() {
        await node.clean();
    });
});
