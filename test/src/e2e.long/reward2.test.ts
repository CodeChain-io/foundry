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
import { U64 } from "codechain-sdk/lib/core/classes";
import "mocha";
import { aliceAddress, aliceSecret, bobAddress } from "../helper/constants";
import CodeChain from "../helper/spawn";

describe("reward2", function() {
    let nodeA: CodeChain;
    let nodeB: CodeChain;

    beforeEach(async function() {
        nodeA = new CodeChain({
            chain: `${__dirname}/../scheme/solo-block-reward-50.json`,
            argv: ["--author", aliceAddress.toString(), "--force-sealing"]
        });
        nodeB = new CodeChain({
            chain: `${__dirname}/../scheme/solo-block-reward-50.json`,
            argv: ["--author", bobAddress.toString(), "--force-sealing"]
        });

        await Promise.all([nodeA.start(), nodeB.start()]);
    });

    it("alice creates an empty block", async function() {
        await nodeA.sdk.rpc.devel.startSealing();
        expect(
            await nodeA.sdk.rpc.chain.getBalance(aliceAddress)
        ).to.deep.equal(new U64(50));

        await nodeB.connect(nodeA);
        await nodeB.waitBlockNumberSync(nodeA);

        expect(
            await nodeB.sdk.rpc.chain.getBalance(aliceAddress)
        ).to.deep.equal(new U64(50));
    }).timeout(30_000);

    afterEach(async function() {
        if (this.currentTest!.state === "failed") {
            nodeA.keepLogs();
            nodeB.keepLogs();
        }
        await Promise.all([nodeA.clean(), nodeB.clean()]);
    });
});
