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

describe("network1 node test", function() {
    let node: CodeChain;
    before(async function() {
        node = new CodeChain();
        await node.start();
    });

    it(`default whitelist [], disabled`, async function() {
        const { list, enabled } = await node.rpc.net.getWhitelist();
        expect(list).to.be.empty;
        expect(enabled).to.equal(false);
    });

    it("default blacklist [], disabled", async function() {
        const { list, enabled } = await node.rpc.net.getBlacklist();
        expect(list).to.be.empty;
        expect(enabled).to.equal(false);
    });

    it("addToWhiteList and removeFromWhitelist", async function() {
        const target = "2.2.2.2";

        await node.rpc.net.addToWhitelist({
            address: target,
            tag: "tag string for the target"
        });
        let { list } = await node.rpc.net.getWhitelist();
        expect(list).to.deep.include([
            "2.2.2.2/32",
            "tag string for the target"
        ]);

        await node.rpc.net.removeFromWhitelist({ address: target });
        ({ list } = await node.rpc.net.getWhitelist());
        expect(list).not.to.include(target);
    });

    it("addToBlacklist and removeFromBlacklist", async function() {
        const target = "1.1.1.1";

        await node.rpc.net.addToBlacklist({
            address: target,
            tag: "tag string for the target"
        });
        let { list } = await node.rpc.net.getBlacklist();
        expect(list).to.deep.include([
            "1.1.1.1/32",
            "tag string for the target"
        ]);

        await node.rpc.net.removeFromBlacklist({ address: target });
        ({ list } = await node.rpc.net.getBlacklist());
        expect(list).not.to.include(target);
    });

    it("enableWhitelist and disableWhitelist", async function() {
        await node.rpc.net.enableWhitelist();
        let { enabled } = await node.rpc.net.getWhitelist();
        expect(enabled).to.be.true;

        await node.rpc.net.disableWhitelist();
        ({ enabled } = await node.rpc.net.getWhitelist());
        expect(enabled).to.be.false;
    });

    it("enableBlacklist and disableBlacklist", async function() {
        await node.rpc.net.enableBlacklist();
        let { enabled } = await node.rpc.net.getBlacklist();
        expect(enabled).to.be.true;

        await node.rpc.net.disableBlacklist();
        ({ enabled } = await node.rpc.net.getBlacklist());
        expect(enabled).to.be.false;
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
