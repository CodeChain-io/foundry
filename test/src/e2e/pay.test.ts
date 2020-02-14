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

    afterEach(function() {
        if (this.currentTest!.state === "failed") {
            node.keepLogs();
        }
    });

    after(async function() {
        await node.clean();
    });
});
