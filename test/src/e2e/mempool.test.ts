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
import { faucetAddress } from "../helper/constants";
import CodeChain from "../helper/spawn";

describe("Sealing test", function() {
    let node: CodeChain;

    beforeEach(async function() {
        node = new CodeChain();
        await node.start();
    });

    it("stopSealing then startSealing", async function() {
        await node.sdk.rpc.devel.stopSealing();
        await node.sendPayTx();
        expect(await node.getBestBlockNumber()).to.equal(0);
        await node.sdk.rpc.devel.startSealing();
        expect(await node.getBestBlockNumber()).to.equal(1);
    });

    afterEach(async function() {
        if (this.currentTest!.state === "failed") {
            node.keepLogs();
        }
        await node.clean();
    });
});

describe("Future queue", function() {
    let node: CodeChain;

    beforeEach(async function() {
        node = new CodeChain();
        await node.start();
    });

    it("all pending transactions must be mined", async function() {
        const seq = (await node.sdk.rpc.chain.getSeq(faucetAddress)) || 0;

        await node.sendPayTx({ seq: seq + 3 });
        expect(await node.sdk.rpc.chain.getSeq(faucetAddress)).to.equal(seq);
        await node.sendPayTx({ seq: seq + 2 });
        expect(await node.sdk.rpc.chain.getSeq(faucetAddress)).to.equal(seq);
        await node.sendPayTx({ seq: seq + 1 });
        expect(await node.sdk.rpc.chain.getSeq(faucetAddress)).to.equal(seq);
        await node.sendPayTx({ seq: seq });
        expect(await node.sdk.rpc.chain.getSeq(faucetAddress)).to.equal(
            seq + 4
        );
    });

    afterEach(async function() {
        if (this.currentTest!.state === "failed") {
            node.keepLogs();
        }
        await node.clean();
    });
});
