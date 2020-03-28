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

describe("CreateShard", function() {
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
        const beforeBlockNumber = await node.rpc.chain.getBestBlockNumber();
        // expect(
        //     node.rpc.chain.getShardIdByHash({
        //         transactionHash: `0x${tx.hash().toString()}`,
        //         blockNumber: null
        //     })
        // ).to.be.null;
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
        const shardUsers = (await node.rpc.chain.getShardUsers({
            shardId
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

    after(async function() {
        await node.clean();
    });
});
