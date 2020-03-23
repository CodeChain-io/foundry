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
import { invalidAddress, invalidSecret } from "../helper/constants";
import { ERROR } from "../helper/error";
import CodeChain from "../helper/spawn";

describe("account", function() {
    describe("account base test", function() {
        let node: CodeChain;
        before(async function() {
            node = new CodeChain();
            await node.start();
        });

        it("getList", async function() {
            expect(await node.rpc.account.getList()).not.to.be.null;
        });

        it("create", async function() {
            expect(await node.rpc.account.create({ passphrase: "my-password" }))
                .not.to.be.null;
            expect(await node.rpc.account.create({ passphrase: "my-password" }))
                .not.to.be.null;
        });

        describe("importRaw", function() {
            let randomSecret: string;
            beforeEach(function() {
                randomSecret = node.testFramework.util.generatePrivateKey();
            });

            it("Ok", async function() {
                const account = node.testFramework.util.getAccountIdFromPrivate(
                    randomSecret
                );
                const address = node.testFramework.core.classes.Address.fromAccountId(
                    account,
                    { networkId: "tc" }
                );
                expect(
                    await node.rpc.account.importRaw({
                        secret: "0x".concat(randomSecret),
                        passphrase: ""
                    })
                ).to.equal(address.toString());
            });

            it("KeyError", async function() {
                try {
                    await node.rpc.account.importRaw({
                        secret: "0x".concat(invalidSecret),
                        passphrase: ""
                    });
                    expect.fail();
                } catch (e) {
                    expect(e).is.similarTo(ERROR.INVALID_SECRET);
                }
            });

            it("AlreadyExists", async function() {
                try {
                    await node.rpc.account.importRaw({
                        secret: "0x".concat(randomSecret),
                        passphrase: null
                    });
                    await node.rpc.account.importRaw({
                        secret: "0x".concat(randomSecret),
                        passphrase: null
                    });
                    expect.fail();
                } catch (e) {
                    expect(e).is.similarTo(ERROR.ALREADY_EXISTS);
                }
            });
        });

        describe("sign", function() {
            const message =
                "0000000000000000000000000000000000000000000000000000000000000000";
            let address: string;
            let secret: string;
            beforeEach(async function() {
                secret = node.testFramework.util.generatePrivateKey();
                address = await node.rpc.account.importRaw({
                    secret: "0x".concat(secret),
                    passphrase: "my-password"
                });
            });

            it("Ok", async function() {
                const calculatedSignature = node.testFramework.util.signEd25519(
                    message,
                    secret
                );
                const signature = await node.rpc.account.sign({
                    message: `0x${message}`,
                    account: address,
                    passphrase: "my-password"
                });
                expect(signature).to.equal(`0x${calculatedSignature}`);
            });

            it("WrongPassword", async function() {
                try {
                    await node.rpc.account.sign({
                        message: `0x${message}`,
                        account: address,
                        passphrase: "wrong-password"
                    });
                    expect.fail();
                } catch (e) {
                    expect(e).is.similarTo(ERROR.WRONG_PASSWORD);
                }
            });

            it("NoSuchAccount", async function() {
                try {
                    await node.rpc.account.sign({
                        message: `0x${message}`,
                        account: invalidAddress,
                        passphrase: "my-password"
                    });
                    expect.fail();
                } catch (e) {
                    expect(e).is.similarTo(ERROR.NO_SUCH_ACCOUNT);
                }
            });
        });

        describe("unlock", function() {
            let address: string;
            beforeEach(async function() {
                address = await node.rpc.account.create({ passphrase: "123" });
            });

            it("Ok", async function() {
                await node.rpc.account.unlock({
                    account: address,
                    passphrase: "123"
                });
                await node.rpc.account.unlock({
                    account: address,
                    passphrase: "123",
                    duration: 0
                });
                await node.rpc.account.unlock({
                    account: address,
                    passphrase: "123",
                    duration: 300
                });
            });

            it("WrongPassword", async function() {
                try {
                    await node.rpc.account.unlock({
                        account: address,
                        passphrase: "456"
                    });
                    expect.fail();
                } catch (e) {
                    expect(e).is.similarTo(ERROR.WRONG_PASSWORD);
                }
            });

            it("NoSuchAccount", async function() {
                try {
                    await node.rpc.account.unlock({
                        account: invalidAddress.toString(),
                        passphrase: "456"
                    });
                    expect.fail();
                } catch (e) {
                    expect(e).is.similarTo(ERROR.NO_SUCH_ACCOUNT);
                }
            });
        });

        describe("changePassword", function() {
            let address: string;
            beforeEach(async function() {
                address = await node.rpc.account.create({ passphrase: "123" });
            });

            it("Ok", async function() {
                await node.rpc.account.changePassword({
                    account: address,
                    oldPassphrase: "123",
                    newPassphrase: "456"
                });
            });

            it("WrongPassword", async function() {
                try {
                    await node.rpc.account.changePassword({
                        account: address,
                        oldPassphrase: "456",
                        newPassphrase: "123"
                    });
                    expect.fail();
                } catch (e) {
                    expect(e).is.similarTo(ERROR.WRONG_PASSWORD);
                }
            });

            it("NoSuchAccount", async function() {
                try {
                    await node.rpc.account.changePassword({
                        account: invalidAddress,
                        oldPassphrase: "123",
                        newPassphrase: "345"
                    });
                    expect.fail();
                } catch (e) {
                    expect(e).is.similarTo(ERROR.NO_SUCH_ACCOUNT);
                }
            });
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
});
