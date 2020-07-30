// Copyright 2020 Kodebox, Inc.
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
import "mocha";
import { PromiseExpect, wait } from "../helper/promise";
import CodeChain from "../helper/spawn";

describe("Informer", function() {
    let nodeA: CodeChain;
    let nodeB: CodeChain;
    const address = "127.0.0.1";
    const promiseExpect = new PromiseExpect();
    before(async function() {
        nodeA = new CodeChain();
        nodeB = new CodeChain();
        await Promise.all([nodeA.start(), nodeB.start()]);
    });
    describe("Hot Subscription", function() {
        it("Client Subscribed", async function() {
            const subscription = nodeA.informerClient();
            subscription.addEventListener("open", () => {
                subscription.send(
                    JSON.stringify({
                        jsonrpc: "2.0",
                        id: 1,
                        method: "register",
                        params: ["PeerAdded"]
                    })
                );
            });
            await promiseExpect.shouldFulfill(
                "on message",
                new Promise(resolve => {
                    subscription.once("message", message => {
                        const json = JSON.parse(message);
                        expect(isFinite(json.result)).to.be.true;
                        resolve();
                    });
                })
            );
        });

        it("Hot Registration", async function() {
            const subscription = nodeA.informerClient();
            subscription.once("open", () => {
                subscription.send(
                    JSON.stringify({
                        jsonrpc: "2.0",
                        id: 1,
                        method: "register",
                        params: ["PeerAdded"]
                    })
                );
            });
            await promiseExpect.shouldFulfill(
                "on message",
                new Promise(resolve => {
                    subscription.addEventListener("message", () => {
                        subscription.once("subscription", message => {
                            const json = JSON.parse(message);
                            expect(isFinite(json.results)).to.be.true;
                        });
                        subscription.once("Get aware", message => {
                            const json = JSON.parse(message);
                            expect(message.result).to.be.includes("PeerAdded");
                        });
                        resolve();
                    });
                })
            );
            await nodeA.rpc.net.connect({
                address: address.toString(),
                port: nodeB.port
            });
            while (
                !(await nodeA.rpc.net.isConnected({
                    address: address.toString(),
                    port: nodeB.port
                }))
            ) {
                await wait(500);
            }
        });
        it("De-registration", async function() {
            const subscription = nodeA.informerClient();
            subscription.once("open", () => {
                subscription.send(
                    JSON.stringify({
                        jsonrpc: "2.0",
                        id: 1,
                        method: "register",
                        params: ["PeerAdded"]
                    })
                );
            });

            let json: any;
            let subscribtionId: string;
            await promiseExpect.shouldFulfill(
                "on message",
                new Promise(resolve => {
                    subscription.addEventListener("message", a => {
                        subscription.once("message", message => {
                            json = JSON.parse(message);
                            subscribtionId = json.result;
                            expect(isFinite(json.result)).to.be.true;
                        });
                        subscription.once("message", message => {
                            json = JSON.parse(message);
                            expect(json.result).to.include("true");
                        });
                        resolve();
                    });
                })
            );

            await nodeA.rpc.net.connect({
                address: address.toString(),
                port: nodeB.port
            });
            while (
                !(await nodeA.rpc.net.isConnected({
                    address: address.toString(),
                    port: nodeB.port
                }))
            ) {
                await wait(500);
            }
            subscription.once("open", () => {
                subscription.send(
                    JSON.stringify({
                        jsonrpc: "2.0",
                        id: 1,
                        method: "deregister",
                        params: [subscribtionId]
                    })
                );
            });
        });
    });

    afterEach(function() {
        if (this.currentTest!.state === "failed") {
            nodeA.keepLogs();
            nodeB.keepLogs();
        }
        promiseExpect.checkFulfilled();
    });

    after(async function() {
        await Promise.all([nodeA.clean(), nodeB.clean()]);
    });
});
