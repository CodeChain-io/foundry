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
import { Address } from "../sdk/core/classes";
import { H160, H256, H512, U64 } from "../sdk/core/classes";
import * as RLP from "rlp";

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
            (await node.rpc.chain.getBlockByHash({ blockHash: blockHash! }))!
                .number
        ).to.equal(bestBlockNumber);
        expect(
            await node.rpc.chain.getBlockByHash({
                blockHash: `0x${invalidH256.toString()}`
            })
        ).to.be.null;
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
        const bestBlockNumber = await node.rpc.chain.getBestBlockNumber();
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
            "tccqxphelyu2n73ekpewrsyj0256wjhn2aqds9xrrrg"
        ];
        expect(accounts.length).to.equal(expected.length);
        expect(accounts).to.include.members(expected);
    });

    it("getPendingTransactions", async function() {
        const pending = await node.rpc.mempool.getPendingTransactions();
        expect(pending.transactions.length).to.equal(0);
    });

    it("sendPayTx, getTransaction", async function() {
        const tx = node.testFramework.core.createPayTransaction({
            recipient: "tccqxv9y4cw0jwphhu65tn4605wadyd2sxu5yezqghw",
            quantity: 0
        });
        const seq = (await node.rpc.chain.getSeq({
            address: faucetAddress.toString(),
            blockNumber: null
        }))!;
        const sig = tx.sign({
            secret: faucetSecret,
            fee: 10,
            seq
        });
        const bytes = sig.rlpBytes().toString("hex");
        const hash = await node.rpc.mempool.sendSignedTransaction({
            tx: `0x${bytes}`
        });
        expect(
            await node.rpc.chain.containsTransaction({
                transactionHash: hash
            })
        ).be.true;
        const thetx = (await node.rpc.chain.getTransaction({
            transactionHash: hash
        }))!;
        expect(thetx).not.null;
        expect(thetx.sig).to.equal(`0x${sig.signature()}`);
        expect(+thetx.fee).to.equal(Number(tx.fee()!.toString()));
    });

    it("sendPayTx, getTransactionSigner", async function() {
        const tx = node.testFramework.core.createPayTransaction({
            recipient: "tccqxv9y4cw0jwphhu65tn4605wadyd2sxu5yezqghw",
            quantity: 0
        });
        const seq = (await node.rpc.chain.getSeq({
            address: faucetAddress.toString(),
            blockNumber: null
        }))!;
        const sig = tx.sign({
            secret: faucetSecret,
            fee: 10,
            seq
        });
        const bytes = sig.rlpBytes().toString("hex");
        const hash = await node.rpc.mempool.sendSignedTransaction({
            tx: `0x${bytes}`
        });
        expect(
            await node.rpc.chain.containsTransaction({ transactionHash: hash })
        ).be.true;
        const signer = await node.rpc.chain.getTransactionSigner({
            transactionHash: hash
        });
        expect(signer).equal(faucetAddress.toString());
        const signed = (await node.rpc.chain.getTransaction({
            transactionHash: hash
        }))!;
        const publicKey = sig.getSignerPublic();
        expect(signed).not.null;
        expect(signed.sig).to.equal(`0x${sig.signature()}`);
        expect(+signed.fee).to.equal(Number(tx.fee()!.toString()));
        expect(
            node.testFramework.core.classes.Address.fromPublic(publicKey, {
                networkId: "tc"
            }).toString()
        ).equal(signer);
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
