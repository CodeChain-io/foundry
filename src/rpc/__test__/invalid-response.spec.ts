import { PlatformAddress } from "codechain-primitives";

import { Pay, U64 } from "../../core/classes";

import { AccountRpc } from "../account";
import { ChainRpc } from "../chain";
import { Rpc } from "../index";
import { NetworkRpc } from "../network";
import { NodeRpc } from "../node";

describe("Invalid response", () => {
    const rpc: Rpc = new Rpc({ server: "" });

    describe("NodeRpc", () => {
        const nodeRpc = new NodeRpc(rpc);
        test("ping", done => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
            nodeRpc
                .ping()
                .then(done.fail)
                .catch(e => {
                    expect(e.toString()).toContain("ping");
                    expect(e.toString()).toContain("string");
                    expect(e.toString()).toContain("undefined");
                    done();
                });
        });

        test("getNodeVersion", done => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
            nodeRpc
                .getNodeVersion()
                .then(done.fail)
                .catch(e => {
                    expect(e.toString()).toContain("getNodeVersion");
                    expect(e.toString()).toContain("string");
                    expect(e.toString()).toContain("undefined");
                    done();
                });
        });

        test("getCommitHash", done => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
            nodeRpc
                .getCommitHash()
                .then(done.fail)
                .catch(e => {
                    expect(e.toString()).toContain("getCommitHash");
                    expect(e.toString()).toContain("string");
                    expect(e.toString()).toContain("undefined");
                    done();
                });
        });
    });

    describe("ChainRpc", () => {
        const chainRpc = new ChainRpc(rpc, {});
        const address = PlatformAddress.fromAccountId(
            "0x0000000000000000000000000000000000000000",
            { networkId: "tc" }
        );
        const hash =
            "0x0000000000000000000000000000000000000000000000000000000000000000";
        const regularKey =
            "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

        describe("sendSignedTransaction", () => {
            const secret =
                "0x0000000000000000000000000000000000000000000000000000000000000001";
            const signedTransaction = new Pay(address, new U64(0), "tc").sign({
                secret,
                fee: 0,
                seq: 0
            });

            test("null", done => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(null);
                chainRpc
                    .sendSignedTransaction(signedTransaction)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain("sendSignedTransaction");
                        expect(e.toString()).toContain("H256");
                        expect(e.toString()).toContain("null");
                        done();
                    });
            });

            test("empty string", done => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce("");
                chainRpc
                    .sendSignedTransaction(signedTransaction)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain("sendSignedTransaction");
                        expect(e.toString()).toContain("H256");
                        expect(e.toString()).toContain(`""`);
                        done();
                    });
            });
        });

        describe("getTransactionById", () => {
            test("undefined", done => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
                chainRpc
                    .getTransaction(hash)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain("chain_getTransaction");
                        expect(e.toString()).toContain("SignedTransaction");
                        expect(e.toString()).toContain("undefined");
                        done();
                    });
            });

            test.skip("Invalid signature", done =>
                done.fail("not implemented"));
            test.skip("Invalid seq", done => done.fail("not implemented"));
            test.skip("Invalid fee", done => done.fail("not implemented"));
            test.skip("Invalid networkId", done =>
                done.fail("not implemented"));
            test.skip("Invalid type", done => done.fail("not implemented"));
        });

        describe("getInvoice", () => {
            test("undefined", done => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
                chainRpc
                    .getInvoice(hash)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain("chain_getInvoice");
                        expect(e.toString()).toContain("JSON of Invoice");
                        expect(e.toString()).toContain("undefined");
                        done();
                    });
            });

            test.skip("Invalid invoice", done => done.fail("not implemented"));
            test.skip("Invalid invoices", done => done.fail("not implemented"));
        });

        test("getRegularKey", done => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
            chainRpc
                .getRegularKey(address)
                .then(() => done.fail())
                .catch(e => {
                    expect(e.toString()).toContain("chain_getRegularKey");
                    expect(e.toString()).toContain("H512");
                    expect(e.toString()).toContain("undefined");
                    done();
                });
        });

        test("getRegularKeyOwner", done => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
            chainRpc
                .getRegularKeyOwner(regularKey)
                .then(() => done.fail())
                .catch(e => {
                    expect(e.toString()).toContain("chain_getRegularKeyOwner");
                    expect(e.toString()).toContain("PlatformAddress");
                    expect(e.toString()).toContain("undefined");
                    done();
                });
        });

        describe("getTransactionById", () => {
            test("undefined", done => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
                chainRpc
                    .getTransactionByTracker(hash)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain("chain_getTransaction");
                        expect(e.toString()).toContain("JSON of Transaction");
                        expect(e.toString()).toContain("undefined");
                        done();
                    });
            });

            test.skip("Invalid type", done => done.fail("not implemented"));
            describe("Invalid data", () => {
                test.skip("AssetMint", done => done.fail("not implemented"));
                test.skip("AssetTransfer", done =>
                    done.fail("not implemented"));
            });
        });

        describe("getInvoicesByTracker", () => {
            test("undefined", done => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce([]);
                chainRpc
                    .getInvoicesByTracker(hash)
                    .then(invoices => {
                        expect(invoices).toEqual([]);
                        done();
                    })
                    .catch(e => {
                        expect(e.toString()).toContain("chain_getInvoice");
                        expect(e.toString()).toContain("JSON of Invoice");
                        expect(e.toString()).toContain("undefined");
                        done();
                    });
            });

            test.skip("Invalid invoice", done => done.fail("not implemented"));
        });

        test("getSeq", done => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
            chainRpc
                .getSeq(address)
                .then(() => done.fail())
                .catch(e => {
                    expect(e.toString()).toContain("chain_getSeq");
                    expect(e.toString()).toContain("undefined");
                    done();
                });
        });

        test("getBalance", done => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
            chainRpc
                .getBalance(address)
                .then(() => done.fail())
                .catch(e => {
                    expect(e.toString()).toContain("chain_getBalance");
                    expect(e.toString()).toContain("U64");
                    expect(e.toString()).toContain("undefined");
                    done();
                });
        });

        test("getBestBlockNumber", done => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
            chainRpc
                .getBestBlockNumber()
                .then(() => done.fail())
                .catch(e => {
                    expect(e.toString()).toContain("chain_getBestBlockNumber");
                    expect(e.toString()).toContain("number");
                    expect(e.toString()).toContain("undefined");
                    done();
                });
        });

        test("getBlockHash", done => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
            chainRpc
                .getBlockHash(0)
                .then(() => done.fail())
                .catch(e => {
                    expect(e.toString()).toContain("chain_getBlockHash");
                    expect(e.toString()).toContain("H256");
                    expect(e.toString()).toContain("undefined");
                    done();
                });
        });

        describe("getBlock", () => {
            test("undefined", done => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
                chainRpc
                    .getBlock(0)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain("chain_getBlock");
                        expect(e.toString()).toContain("JSON of Block");
                        expect(e.toString()).toContain("undefined");
                        done();
                    });
            });

            test.skip("Invalid timestamp", done =>
                done.fail("not implemented"));
            test.skip("Invalid number", done => done.fail("not implemented"));
        });

        describe("getAssetSchemeByTracker", () => {
            test("undefined", done => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
                chainRpc
                    .getAssetSchemeByTracker(hash, 0)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain(
                            "chain_getAssetSchemeByTracker"
                        );
                        expect(e.toString()).toContain("JSON of AssetScheme");
                        expect(e.toString()).toContain("undefined");
                        done();
                    });
            });

            test.skip("Invalid asset scheme", done =>
                done.fail("not implemented"));
        });

        describe("getAssetSchemeByType", () => {
            test("undefined", done => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
                chainRpc
                    .getAssetSchemeByType(hash)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain(
                            "chain_getAssetSchemeByType"
                        );
                        expect(e.toString()).toContain("JSON of AssetScheme");
                        expect(e.toString()).toContain("undefined");
                        done();
                    });
            });

            test.skip("Invalid asset scheme", done =>
                done.fail("not implemented"));
        });

        describe("getAsset", () => {
            test("undefined", done => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
                chainRpc
                    .getAsset(hash, 0)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain("chain_getAsset");
                        expect(e.toString()).toContain("JSON of Asset");
                        expect(e.toString()).toContain("undefined");
                        done();
                    });
            });

            test.skip("Invalid asset", done => done.fail("not implemented"));
        });

        test("isAssetSpent", done => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
            chainRpc
                .isAssetSpent(hash, 0, 0)
                .then(() => done.fail())
                .catch(e => {
                    expect(e.toString()).toContain("chain_isAssetSpent");
                    expect(e.toString()).toContain("boolean");
                    expect(e.toString()).toContain("undefined");
                    done();
                });
        });

        describe("getPendingTransactions", () => {
            test("undefined", done => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
                chainRpc
                    .getPendingTransactions()
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain(
                            "chain_getPendingTransactions"
                        );
                        expect(e.toString()).toContain("array");
                        expect(e.toString()).toContain("undefined");
                        done();
                    });
            });

            test.skip("Invalid transactions", done =>
                done.fail("not implemented"));
        });

        test("getNetworkId", done => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
            chainRpc
                .getNetworkId()
                .then(() => done.fail())
                .catch(e => {
                    expect(e.toString()).toContain("chain_getNetworkId");
                    expect(e.toString()).toContain("string");
                    expect(e.toString()).toContain("undefined");
                    done();
                });
        });
    });

    describe("AccountRpc", () => {
        const accountRpc = new AccountRpc(rpc, {});
        const hash =
            "0x0000000000000000000000000000000000000000000000000000000000000000";
        const secret =
            "0x0000000000000000000000000000000000000000000000000000000000000001";
        const address = PlatformAddress.fromAccountId(
            "0x0000000000000000000000000000000000000000",
            { networkId: "tc" }
        );

        describe("getList", () => {
            test("undefined", done => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
                accountRpc
                    .getList()
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain("account_getList");
                        expect(e.toString()).toContain("array");
                        expect(e.toString()).toContain("undefined");
                        done();
                    });
            });

            test.skip("invalid address string", done =>
                done.fail("not implemented"));
        });

        describe("create", () => {
            test("undefined", done => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
                accountRpc
                    .create()
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain("account_create");
                        expect(e.toString()).toContain("PlatformAddress");
                        expect(e.toString()).toContain("undefined");
                        done();
                    });
            });

            test.skip("invalid address string", done =>
                done.fail("not implemented"));
        });

        describe("importRaw", () => {
            test("undefined", done => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
                accountRpc
                    .importRaw(secret)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain("account_importRaw");
                        expect(e.toString()).toContain("PlatformAddress");
                        expect(e.toString()).toContain("undefined");
                        done();
                    });
            });

            test.skip("invalid address string", done =>
                done.fail("not implemented"));
        });

        describe("sign", () => {
            test("undefined", done => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
                accountRpc
                    .sign(hash, address)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain("account_sign");
                        expect(e.toString()).toContain("string");
                        expect(e.toString()).toContain("undefined");
                        done();
                    });
            });

            test.skip("invalid signature string", done =>
                done.fail("not implemented"));
        });

        test("unlock", done => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
            accountRpc
                .unlock(address)
                .then(() => done.fail())
                .catch(e => {
                    expect(e.toString()).toContain("account_unlock");
                    expect(e.toString()).toContain("null");
                    expect(e.toString()).toContain("undefined");
                    done();
                });
        });
    });

    describe("NetworkRpc", () => {
        const networkRpc = new NetworkRpc(rpc);

        test.skip("shareSecret", done => done.fail("not implemented"));
        test.skip("connect", done => done.fail("not implemented"));
        test.skip("disconnect", done => done.fail("not implemented"));
        test.skip("isConnected", done => done.fail("not implemented"));
        test.skip("getPeerCount", done => done.fail("not implemented"));
        describe("getPeers", () => {
            test("undefined", done => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
                networkRpc
                    .getPeers()
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain(
                            "net_getEstablishedPeers"
                        );
                        expect(e.toString()).toContain("array");
                        expect(e.toString()).toContain("undefined");
                        done();
                    });
            });

            test("invalid address", done => {
                rpc.sendRpcRequest = jest
                    .fn()
                    .mockResolvedValueOnce([
                        "127.0.0.1:3333",
                        "127.0.0.1111:3333"
                    ]);
                networkRpc
                    .getPeers()
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain(
                            "net_getEstablishedPeers"
                        );
                        expect(e.toString()).toContain("IPv4");
                        expect(e.toString()).toContain("127.0.0.1111:3333");
                        done();
                    });
            });
        });
        test.skip("addToWhitelist", done => done.fail("not implemented"));
        test.skip("removeFromWhitelist", done => done.fail("not implemented"));
        test.skip("addToBlocklist", done => done.fail("not implemented"));
        test.skip("removeFromBlockList", done => done.fail("not implemented"));
        test.skip("enableWhitelist", done => done.fail("not implemented"));
        test.skip("disableWhitelist", done => done.fail("not implemented"));
        test.skip("enableBlacklist", done => done.fail("not implemented"));
        test.skip("disableBlacklist", done => done.fail("not implemented"));
        test.skip("getWhitelist", done => done.fail("not implemented"));
        test.skip("getBlacklist", done => done.fail("not implemented"));
    });
});
