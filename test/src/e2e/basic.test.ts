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
import CodeChain from "../helper/spawn";
import { log } from "util";

describe("solo - 1 node", function() {
    let node: CodeChain;
    before(async function() {
        node = new CodeChain();
        await node.start();
    });

    it("ping", async function() {
        expect(await node.rpc.ping()).to.contain("pong");
    });

    it("getNodeVersion", async function() {
        expect(await node.rpc.version()).to.match(
            /^[0-9]+\.[0-9]+\.[0-9]+(-[a-z0-9.]*)?$/
        );
    });

    it("getCommitHash", async function() {
        expect(await node.rpc.commitHash()).to.match(/^[a-fA-F0-9]{40}$/);
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
