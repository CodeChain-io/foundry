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
import "mocha";
import { PlatformAddress } from "codechain-primitives/lib";
import { aliceAddress, aliceSecret, faucetAddress } from "../helper/constants";
import CodeChain from "../helper/spawn";

describe("Pay", async function() {
    let node: CodeChain;
    before(async function() {
        node = new CodeChain();
        await node.start();
    });

    it("Allow zero pay", async function() {
        const pay = await node.sendPayTx({ quantity: 0 });
        expect(
            await node.rpc.chain.containsTransaction({
                transactionHash: `0x${pay.hash().toString()}`
            })
        ).be.true;
        expect(
            await node.rpc.chain.getTransaction({
                transactionHash: `0x${pay.hash().toString()}`
            })
        ).not.null;
    });

    it("Allow pay to itself", async function() {
        const pay = await node.sendPayTx({
            quantity: 100,
            recipient: faucetAddress
        });
        expect(
            await node.rpc.chain.containsTransaction({
                transactionHash: `0x${pay.hash().toString()}`
            })
        ).be.true;
        expect(
            await node.rpc.chain.getTransaction({
                transactionHash: `0x${pay.hash().toString()}`
            })
        ).not.null;
    });

    it("Cannot pay to regular key", async function() {
        const charge = await node.sendPayTx({
            quantity: 100000,
            recipient: aliceAddress
        });
        expect(
            await node.rpc.chain.containsTransaction({
                transactionHash: `0x${charge.hash().toString()}`
            })
        ).be.true;
        expect(
            await node.rpc.chain.getTransaction({
                transactionHash: `0x${charge.hash().toString()}`
            })
        ).not.null;

        const privKey = node.sdk.util.generatePrivateKey();
        const pubKey = node.sdk.util.getPublicFromPrivate(privKey);
        const aliceSeq = (await node.rpc.chain.getSeq({
            address: aliceAddress.toString(),
            blockNumber: null
        }))!;
        await node.setRegularKey(pubKey, {
            seq: aliceSeq,
            secret: aliceSecret
        });
        const addressOfRegularKey = PlatformAddress.fromPublic(pubKey, {
            networkId: node.sdk.networkId
        });

        const seq = (await node.rpc.chain.getSeq({
            address: faucetAddress.toString(),
            blockNumber: null
        }))!;
        const blockNumber = await node.getBestBlockNumber();

        await node.rpc.devel!.stopSealing();

        const pay = await node.sendPayTx({ quantity: 0, seq });
        const fail = await node.sendPayTx({
            quantity: 100000,
            recipient: addressOfRegularKey,
            seq: seq + 1
        });

        await node.rpc.devel!.startSealing();
        await node.waitBlockNumber(blockNumber + 1);

        expect(
            await node.rpc.chain.containsTransaction({
                transactionHash: `0x${charge.hash().toString()}`
            })
        ).be.true;
        expect(
            await node.rpc.chain.getTransaction({
                transactionHash: `0x${pay.hash().toString()}`
            })
        ).not.null;

        expect(
            await node.rpc.chain.containsTransaction({
                transactionHash: `0x${fail.hash().toString()}`
            })
        ).be.false;
        expect(
            await node.rpc.chain.getTransaction({
                transactionHash: `0x${fail.hash().toString()}`
            })
        ).null;
        expect(await node.sdk.rpc.chain.getErrorHint(fail.hash())).not.null;
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
