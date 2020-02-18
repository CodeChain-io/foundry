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

import * as chai from "chai";
import * as chaiAsPromised from "chai-as-promised";
chai.use(chaiAsPromised);
const expect = chai.expect;
import "mocha";
import {
    faucetAddress,
    faucetSecret,
    invalidAddress
} from "../helper/constants";
import CodeChain from "../helper/spawn";
import { H160, H256, U64 } from "../sdk/src/core/classes";
const RLP = require("rlp");

describe("chain", function() {
    const invalidH160 = H160.zero();
    const invalidH256 = H256.zero();

    let node: CodeChain;
    before(async function() {
        node = new CodeChain();
        await node.start();
    });

    it("getNetworkId", async function() {
        expect(await node.rpc.chain.getNetworkId()).to.equal("tc");
    });

    it("getBestBlockNumber", async function() {
        expect(await node.rpc.chain.getBestBlockNumber()).to.be.a("number");
    });

    it("getPossibleAuthors", async function() {
        expect(await node.rpc.chain.getPossibleAuthors({ blockNumber: null }))
            .be.null;
    });

    it("getPossibleAuthors of the genesis block", async function() {
        expect(
            await node.rpc.chain.getPossibleAuthors({ blockNumber: 0 })
        ).deep.equal(["tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhhn9p3"]);
    });

    it("getBestBlockId", async function() {
        const value = await node.rpc.chain.getBestBlockNumber();
        expect(value).to.be.a("number");
    });

    it("getBlockHash", async function() {
        const bestBlockNumber = await node.rpc.chain.getBestBlockNumber();
        expect(
            await node.rpc.chain.getBlockHash({ blockNumber: bestBlockNumber })
        ).not.to.be.null;
        expect(
            await node.rpc.chain.getBlockHash({
                blockNumber: bestBlockNumber + 1
            })
        ).to.be.null;
    });

    it("getBlockByHash", async function() {
        const bestBlockNumber = await node.rpc.chain.getBestBlockNumber();
        const blockHash = await node.rpc.chain.getBlockHash({
            blockNumber: bestBlockNumber
        });
        expect(
            (await node.sdk.rpc.chain.getBlock(blockHash!))!.number
        ).to.equal(bestBlockNumber);
        expect(await node.sdk.rpc.chain.getBlock(invalidH256)).to.be.null;
    });

    it("getSeq", async function() {
        await node.rpc.chain.getSeq({
            address: faucetAddress.toString(),
            blockNumber: null
        });
        expect(
            await node.rpc.chain.getSeq({
                address: invalidAddress,
                blockNumber: null
            })
        ).to.equal(0);
        const bestBlockNumber = await node.sdk.rpc.chain.getBestBlockNumber();
        await node.rpc.chain.getSeq({
            address: faucetAddress.toString(),
            blockNumber: 0
        });
        await node.rpc.chain.getSeq({
            address: faucetAddress.toString(),
            blockNumber: bestBlockNumber
        });
        await expect(
            node.rpc.chain.getSeq({
                address: faucetAddress.toString(),
                blockNumber: bestBlockNumber + 1
            })
        ).to.be.empty;
    });

    it("getBalance", async function() {
        await node.rpc.chain.getBalance({
            address: faucetAddress.toString(),
            blockNumber: null
        });
        expect(
            await node.rpc.chain.getBalance({
                address: invalidAddress.toString(),
                blockNumber: null
            })
        ).to.deep.include(new U64(0));
        const bestBlockNumber = await node.rpc.chain.getBestBlockNumber();
        await node.rpc.chain.getBalance({
            address: faucetAddress.toString(),
            blockNumber: 0
        });
        await node.rpc.chain.getBalance({
            address: faucetAddress.toString(),
            blockNumber: bestBlockNumber
        });
        await node.rpc.chain.getBalance({
            address: faucetAddress.toString(),
            blockNumber: bestBlockNumber + 1
        });
    });

    it("getGenesisAccounts", async function() {
        // FIXME: Add an API to SDK
        const accounts = await node.rpc.chain.getGenesisAccounts();
        const expected = [
            "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqyca3rwt",
            "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqgfrhflv",
            "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqvxf40sk",
            "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqszkma5z",
            "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq5duemmc",
            "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqcuzl32l",
            "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqungah99",
            "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqpqc2ul2h",
            "tccq8vapdlstar6ghmqgczp6j2e83njsqq0tsvaxm9u",
            "tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd"
        ];
        expect(accounts.length).to.equal(expected.length);
        expect(accounts).to.include.members(expected);
    });

    it("getBlockReward", async function() {
        // FIXME: Add an API to SDK
        const reward = await node.sdk.rpc.sendRpcRequest(
            "engine_getBlockReward",
            [10]
        );
        expect(reward).to.equal(0);
    });

    it("getPendingTransactions", async function() {
        const pending = await node.sdk.rpc.chain.getPendingTransactions();
        expect(pending.transactions.length).to.equal(0);
    });

    it("sendPayTx, getTransaction", async function() {
        const tx = node.sdk.core.createPayTransaction({
            recipient: "tccqxv9y4cw0jwphhu65tn4605wadyd2sxu5yezqghw",
            quantity: 0
        });
        const seq = (await node.rpc.chain.getSeq({
            address: faucetAddress.toString(),
            blockNumber: null
        }))!;
        const hash = await node.sdk.rpc.chain.sendSignedTransaction(
            tx.sign({
                secret: faucetSecret,
                fee: 10,
                seq
            })
        );
        expect(
            await node.rpc.chain.containsTransaction({
                transactionHash: "0x".concat(hash.toString())
            })
        ).be.true;
        const signed = await node.sdk.rpc.chain.getTransaction(hash);
        expect(signed).not.null;
        expect(signed!.unsigned).to.deep.equal(tx);
    });

    it("sendPayTx, getTransactionSigner", async function() {
        const tx = node.sdk.core.createPayTransaction({
            recipient: "tccqxv9y4cw0jwphhu65tn4605wadyd2sxu5yezqghw",
            quantity: 0
        });
        const seq = (await node.rpc.chain.getSeq({
            address: faucetAddress.toString(),
            blockNumber: null
        }))!;
        const hash = await node.sdk.rpc.chain.sendSignedTransaction(
            tx.sign({
                secret: faucetSecret,
                fee: 10,
                seq
            })
        );
        expect(await node.sdk.rpc.chain.containsTransaction(hash)).be.true;
        const signer = await node.sdk.rpc.sendRpcRequest(
            "chain_getTransactionSigner",
            [hash]
        );
        expect(signer).equal(faucetAddress.toString());
        const signed = await node.sdk.rpc.chain.getTransaction(hash);
        expect(signed).not.null;
        expect(signed!.unsigned).to.deep.equal(tx);
        expect(
            node.sdk.core.classes.PlatformAddress.fromPublic(
                signed!.getSignerPublic(),
                { networkId: "tc" }
            ).toString()
        ).equal(signer);
    });

    it("getNumberOfShards", async function() {
        expect(await node.rpc.chain.getNumberOfShards()).to.equal(1);

        expect(
            await node.rpc.chain.getNumberOfShards({ blockNumber: 0 })
        ).to.equal(1);
    });

    it("getShardRoot", async function() {
        const result1 = (await node.rpc.chain.getShardRoot({
            shardId: 0,
            blockNumber: null
        }))!;
        expect(result1).not.to.be.null;
        H256.ensure(result1);
        const result2 = (await node.rpc.chain.getShardRoot({
            shardId: 0,
            blockNumber: 0
        }))!;
        expect(result2).not.to.be.null;
        H256.ensure(result2);
        expect(
            await node.rpc.chain.getShardRoot({
                shardId: 10000,
                blockNumber: null
            })
        ).to.be.null;
    });

    it("getMiningReward", async function() {
        expect(
            await node.rpc.chain.getMiningReward({ blockNumber: 0 })
        ).to.equal(0);
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
