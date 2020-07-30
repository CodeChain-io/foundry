// Copyright 2018-2019 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

import { expect } from "chai";
import * as fs from "fs";
import "mocha";
import * as path from "path";

import { aliceAddress } from "../helper/constants";
import CodeChain from "../helper/spawn";

describe("Snapshot", async function() {
    let node: CodeChain;
    before(async function() {
        node = new CodeChain();
        await node.start();
    });

    it("can make a snapshot when it is requested with devel rpc", async function() {
        const pay = await node.sendPayTx({
            quantity: 100,
            recipient: aliceAddress
        });

        const blockHash = (await node.rpc.chain.getTransaction({
            transactionHash: `0x${pay.hash().toString()}`
        }))!.blockHash!;
        await node.rpc.devel!.developSnapshot({ hash: blockHash })!;
        // Wait for 1 secs
        await new Promise(resolve => setTimeout(resolve, 1000));

        const stateRoot = (await node.rpc.chain.getBlockByHash({
            blockHash
        }))!.stateRoot;
        expect(
            path.join(
                node.snapshotPath,
                blockHash.substr(2),
                stateRoot.substr(2)
            )
        ).to.satisfies(fs.existsSync);
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
