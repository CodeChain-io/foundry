// Copyright 2020 Kodebox, Inc.
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
import { PromiseExpect, wait } from "../helper/promise";
import CodeChain from "../helper/spawn";

describe("Informer", function() {
    let nodeA: CodeChain;
    const address = "127.0.0.1";
    const promiseExpect = new PromiseExpect();
    before(async function() {
        nodeA = new CodeChain({ argv: ["--force-sealing"] });
        await Promise.all([nodeA.start()]);
    });

    describe("Cold Subscription", async function() {
        it("BlockGeneration", async function() {
            const subscription = nodeA.informerClient();
            subscription.addEventListener("open", () => {
                subscription.send(
                    JSON.stringify({
                        jsonrpc: "2.0",
                        id: 1,
                        method: "register",
                        params: ["BlockGeneration_by_number", "1"]
                    })
                );
            });
            let json: any;
            await promiseExpect.shouldFulfill(
                "on message",
                new Promise(resolve => {
                    subscription.addEventListener("message", a => {
                        subscription.once("message", message => {
                            json = JSON.parse(message);
                            expect(isFinite(json.result)).to.be.true;
                        });
                        subscription.once("message", message => {
                            json = JSON.parse(message);
                            expect(json.result).to.include("BlockGeneration");
                        });
                        resolve();
                    });
                })
            );
            await Promise.all([
                nodeA.rpc.devel!.startSealing(),
                nodeA.rpc.devel!.startSealing(),
                nodeA.rpc.devel!.startSealing()
            ]);
        });
    });

    afterEach(function() {
        if (this.currentTest!.state === "failed") {
            nodeA.keepLogs();
        }
        promiseExpect.checkFulfilled();
    });

    after(async function() {
        await Promise.all([nodeA.clean()]);
    });
});
