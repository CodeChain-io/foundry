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
import { H160, H256, H512, U64 } from "codechain-sdk/lib/core/classes";
import "mocha";
import {
    faucetAddress,
    faucetSecret,
    invalidAddress
} from "../helper/constants";
import CodeChain from "../helper/spawn";

describe("chain", function() {
    const invalidH160 = H160.zero();
    const invalidH256 = H256.zero();

    let node: CodeChain;
    before(async function() {
        node = new CodeChain();
        await node.start();
    });

    it("getNetworkId", async function() {
        expect(await node.sdk.rpc.chain.getNetworkId()).to.equal("tc");
    });

    it("getBestBlockNumber", async function() {
        expect(await node.sdk.rpc.chain.getBestBlockNumber()).to.be.a("number");
    });

    it("getPossibleAuthors", async function() {
        expect(
            await node.sdk.rpc.sendRpcRequest("chain_getPossibleAuthors", [
                null
            ])
        ).be.null;
    });

    it("getPossibleAuthors of the genesis block", async function() {
        expect(
            await node.sdk.rpc.sendRpcRequest("chain_getPossibleAuthors", [0])
        ).deep.equal(["tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhhn9p3"]);
    });

    it("getBestBlockId", async function() {
        const value = await node.sdk.rpc.sendRpcRequest(
            "chain_getBestBlockId",
            []
        );
        expect(value.hash).to.be.a("string");
        new H256(value.hash);
        expect(value.number).to.be.a("number");
    });

    it("getBlockHash", async function() {
        const bestBlockNumber = await node.sdk.rpc.chain.getBestBlockNumber();
        expect(await node.sdk.rpc.chain.getBlockHash(bestBlockNumber)).not.to.be
            .null;
        expect(await node.sdk.rpc.chain.getBlockHash(bestBlockNumber + 1)).to.be
            .null;
    });

    it("getBlockByHash", async function() {
        const bestBlockNumber = await node.sdk.rpc.chain.getBestBlockNumber();
        const blockHash = await node.sdk.rpc.chain.getBlockHash(
            bestBlockNumber
        );
        expect(
            (await node.sdk.rpc.chain.getBlock(blockHash!))!.number
        ).to.equal(bestBlockNumber);
        expect(await node.sdk.rpc.chain.getBlock(invalidH256)).to.be.null;
    });

    it("getSeq", async function() {
        await node.sdk.rpc.chain.getSeq(faucetAddress);
        expect(await node.sdk.rpc.chain.getSeq(invalidAddress)).to.equal(0);
        const bestBlockNumber = await node.sdk.rpc.chain.getBestBlockNumber();
        await node.sdk.rpc.chain.getSeq(faucetAddress, 0);
        await node.sdk.rpc.chain.getSeq(faucetAddress, bestBlockNumber);
        await expect(
            node.sdk.rpc.chain.getSeq(faucetAddress, bestBlockNumber + 1)
        ).to.be.rejectedWith("chain_getSeq returns undefined");
    });

    it("getBalance", async function() {
        await node.sdk.rpc.chain.getBalance(faucetAddress);
        expect(
            await node.sdk.rpc.chain.getBalance(invalidAddress)
        ).to.deep.equal(new U64(0));
        const bestBlockNumber = await node.sdk.rpc.chain.getBestBlockNumber();
        await node.sdk.rpc.chain.getBalance(faucetAddress, 0);
        await node.sdk.rpc.chain.getBalance(faucetAddress, bestBlockNumber);
        await node.sdk.rpc.chain.getBalance(faucetAddress, bestBlockNumber + 1);
    });

    it("getGenesisAccounts", async function() {
        // FIXME: Add an API to SDK
        const accounts = await node.sdk.rpc.sendRpcRequest(
            "chain_getGenesisAccounts",
            []
        );
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
        const seq = await node.sdk.rpc.chain.getSeq(faucetAddress);
        const hash = await node.sdk.rpc.chain.sendSignedTransaction(
            tx.sign({
                secret: faucetSecret,
                fee: 10,
                seq
            })
        );
        expect(await node.sdk.rpc.chain.containsTransaction(hash)).be.true;
        const signed = await node.sdk.rpc.chain.getTransaction(hash);
        expect(signed).not.null;
        expect(signed!.unsigned).to.deep.equal(tx);
    });

    it("sendPayTx, getTransactionSigner", async function() {
        const tx = node.sdk.core.createPayTransaction({
            recipient: "tccqxv9y4cw0jwphhu65tn4605wadyd2sxu5yezqghw",
            quantity: 0
        });
        const seq = await node.sdk.rpc.chain.getSeq(faucetAddress);
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

    it("getRegularKey, getRegularKeyOwner", async function() {
        const key = node.sdk.util.getPublicFromPrivate(
            node.sdk.util.generatePrivateKey()
        );

        const secret = node.sdk.util.generatePrivateKey();
        const accountId = node.sdk.util.getAccountIdFromPrivate(secret);
        const address = node.sdk.core.classes.PlatformAddress.fromAccountId(
            accountId,
            { networkId: "tc" }
        );

        await node.sendPayTx({ quantity: 10000, recipient: address });
        expect(await node.sdk.rpc.chain.getRegularKey(address)).to.be.null;
        expect(await node.sdk.rpc.chain.getRegularKeyOwner(key)).to.be.null;

        const tx = node.sdk.core
            .createSetRegularKeyTransaction({
                key
            })
            .sign({
                secret,
                fee: 10,
                seq: await node.sdk.rpc.chain.getSeq(address)
            });
        await node.sdk.rpc.chain.sendSignedTransaction(tx);

        expect(await node.sdk.rpc.chain.getRegularKey(address)).to.deep.equal(
            new H512(key)
        );
        expect(await node.sdk.rpc.chain.getRegularKeyOwner(key)).to.deep.equal(
            address
        );

        const bestBlockNumber = await node.sdk.rpc.chain.getBestBlockNumber();
        expect(
            await node.sdk.rpc.chain.getRegularKey(address, bestBlockNumber)
        ).to.deep.equal(new H512(key));
        expect(await node.sdk.rpc.chain.getRegularKey(address, 0)).to.be.null;
        expect(
            await node.sdk.rpc.chain.getRegularKey(address, bestBlockNumber + 1)
        ).to.be.null;

        expect(
            await node.sdk.rpc.chain.getRegularKeyOwner(key, bestBlockNumber)
        ).to.deep.equal(address);
        expect(await node.sdk.rpc.chain.getRegularKeyOwner(key, 0)).to.be.null;
        expect(
            await node.sdk.rpc.chain.getRegularKeyOwner(
                key,
                bestBlockNumber + 1
            )
        ).to.be.null;
    });

    it("getNumberOfShards", async function() {
        expect(
            await node.sdk.rpc.sendRpcRequest("chain_getNumberOfShards", [null])
        ).to.equal(1);

        expect(
            await node.sdk.rpc.sendRpcRequest("chain_getNumberOfShards", [0])
        ).to.equal(1);
    });

    it("getShardRoot", async function() {
        await node.sdk.rpc
            .sendRpcRequest("chain_getShardRoot", [0, null])
            .then(result => {
                expect(result).not.to.be.null;
                H256.ensure(result);
            });

        await node.sdk.rpc
            .sendRpcRequest("chain_getShardRoot", [0, 0])
            .then(result => {
                expect(result).not.to.be.null;
                H256.ensure(result);
            });

        await node.sdk.rpc
            .sendRpcRequest("chain_getShardRoot", [10000, null])
            .then(result => {
                expect(result).to.be.null;
            });
    });

    it("getMiningReward", async function() {
        await node.sdk.rpc
            .sendRpcRequest("chain_getMiningReward", [0])
            .then(result => {
                expect(result).to.equal(0);
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
