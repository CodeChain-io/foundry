import { Rpc } from "../index";
import { NodeRpc } from "../node";
import { ChainRpc } from "../chain";
import { PlatformAddress } from "../../key/classes";
import { Parcel, Payment, U256 } from "../../core/classes";

describe("Invalid response", () => {
    const rpc: Rpc = new Rpc({ server: "" });

    describe("NodeRpc", () => {
        const nodeRpc = new NodeRpc(rpc);
        test("ping", (done) => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
            nodeRpc.ping()
                .then(done.fail)
                .catch(e => {
                    expect(e).toEqual(Error("Expected ping() to return a string but undefined is given"));
                    done();
                });
        });

        test("getNodeVersion", (done) => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
            nodeRpc.getNodeVersion()
                .then(done.fail)
                .catch(e => {
                    expect(e).toEqual(Error("Expected getNodeVersion() to return a string but undefined is given"));
                    done();
                });
        });

        test("getCommitHash", (done) => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
            nodeRpc.getCommitHash()
                .then(done.fail)
                .catch(e => {
                    expect(e).toEqual(Error("Expected getCommitHash() to return a string but undefined is given"));
                    done();
                });
        });
    });

    describe("ChainRpc", () => {
        const chainRpc = new ChainRpc(rpc, {});
        const address = PlatformAddress.fromAccountId("0x0000000000000000000000000000000000000000");
        const hash = "0x0000000000000000000000000000000000000000000000000000000000000000";

        describe("sendSignedParcel", () => {
            const secret = "0x0000000000000000000000000000000000000000000000000000000000000001";
            const signedParcel = new Parcel("tc", new Payment(address, new U256(0))).sign({
                secret,
                fee: 0,
                nonce: 0
            });

            test("null", (done) => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(null);
                chainRpc.sendSignedParcel(signedParcel)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain("sendSignedParcel");
                        expect(e.toString()).toContain("H256");
                        expect(e.toString()).toContain("null");
                        done();
                    });
            });

            test("empty string", (done) => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce("");
                chainRpc.sendSignedParcel(signedParcel)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain("sendSignedParcel");
                        expect(e.toString()).toContain("H256");
                        expect(e.toString()).toContain(`""`);
                        done();
                    });
            });
        });

        describe("getParcel", () => {
            test("undefined", (done) => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
                chainRpc.getParcel(hash)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain("chain_getParcel");
                        expect(e.toString()).toContain("SignedParcel");
                        expect(e.toString()).toContain("undefined");
                        done();
                    });
            });

            test.skip("Invalid signature", (done) => done.fail("not implemented"));
            test.skip("Invalid nonce", (done) => done.fail("not implemented"));
            test.skip("Invalid fee", (done) => done.fail("not implemented"));
            test.skip("Invalid networkId", (done) => done.fail("not implemented"));
            test.skip("Invalid action", (done) => done.fail("not implemented"));
        });

        describe("getParcelInvoice", () => {
            test("undefined", (done) => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
                chainRpc.getParcelInvoice(hash)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain("chain_getParcelInvoice");
                        expect(e.toString()).toContain("JSON of Invoice");
                        expect(e.toString()).toContain("undefined");
                        done();
                    });
            });

            test.skip("Invalid invoice", (done) => done.fail("not implemented"));
            test.skip("Invalid invoices", (done) => done.fail("not implemented"));
        });

        test("getRegularKey", (done) => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(null);
            chainRpc.getRegularKey(address)
                .then(() => done.fail())
                .catch(e => {
                    expect(e.toString()).toContain("chain_getRegularKey");
                    expect(e.toString()).toContain("H512");
                    expect(e.toString()).toContain("null");
                    done();
                });
        });

        describe("getTransaction", () => {
            test("undefined", (done) => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
                chainRpc.getTransaction(hash)
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
                test.skip("CreateWorld", done => done.fail("not implemented"));
                test.skip("SetWorldOwner", done => done.fail("not implemented"));
                test.skip("SetWorldUsers", done => done.fail("not implemented"));
                test.skip("AssetMint", done => done.fail("not implemented"));
                test.skip("AssetTransfer", done => done.fail("not implemented"));
            });
        });

        describe("getTransactionInvoice", () => {
            test("undefined", (done) => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
                chainRpc.getTransactionInvoice(hash)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain("chain_getTransactionInvoice");
                        expect(e.toString()).toContain("JSON of Invoice");
                        expect(e.toString()).toContain("undefined");
                        done();
                    });
            });

            test.skip("Invalid invoice", done => done.fail("not implemented"));
        });

        test("getNonce", (done) => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(null);
            chainRpc.getNonce(address)
                .then(() => done.fail())
                .catch(e => {
                    expect(e.toString()).toContain("chain_getNonce");
                    expect(e.toString()).toContain("U256");
                    expect(e.toString()).toContain("null");
                    done();
                });
        });

        test("getBalance", (done) => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(null);
            chainRpc.getBalance(address)
                .then(() => done.fail())
                .catch(e => {
                    expect(e.toString()).toContain("chain_getBalance");
                    expect(e.toString()).toContain("U256");
                    expect(e.toString()).toContain("null");
                    done();
                });
        });

        test("getBestBlockNumber", (done) => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
            chainRpc.getBestBlockNumber()
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
            chainRpc.getBlockHash(0)
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
                chainRpc.getBlock(0)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain("chain_getBlock");
                        expect(e.toString()).toContain("JSON of Block");
                        expect(e.toString()).toContain("undefined");
                        done();
                    });
            });

            test.skip("Invalid timestamp", done => done.fail("not implemented"));
            test.skip("Invalid number", done => done.fail("not implemented"));
        });

        describe("getAssetSchemeByHash", () => {
            test("undefined", done => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
                chainRpc.getAssetSchemeByHash(hash, 0, 0)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain("chain_getAssetSchemeByHash");
                        expect(e.toString()).toContain("JSON of AssetScheme");
                        expect(e.toString()).toContain("undefined");
                        done();
                    });
            });

            test.skip("Invalid asset scheme", done => done.fail("not implemented"));
        });

        describe("getAssetSchemeByType", () => {
            test("undefined", done => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
                chainRpc.getAssetSchemeByType(hash)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain("chain_getAssetSchemeByType");
                        expect(e.toString()).toContain("JSON of AssetScheme");
                        expect(e.toString()).toContain("undefined");
                        done();
                    });
            });

            test.skip("Invalid asset scheme", done => done.fail("not implemented"));
        });

        describe("getAsset", () => {
            test("undefined", done => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
                chainRpc.getAsset(hash, 0)
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
            chainRpc.isAssetSpent(hash, 0, 0)
                .then(() => done.fail())
                .catch(e => {
                    expect(e.toString()).toContain("chain_isAssetSpent");
                    expect(e.toString()).toContain("boolean");
                    expect(e.toString()).toContain("undefined");
                    done();
                });
        });

        describe("getPendingParcels", () => {
            test("undefined", done => {
                rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
                chainRpc.getPendingParcels()
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e.toString()).toContain("chain_getPendingParcels");
                        expect(e.toString()).toContain("array");
                        expect(e.toString()).toContain("undefined");
                        done();
                    });
            });

            test.skip("Invalid parcels", done => done.fail("not implemented"));
        });

        test("getNetworkId", done => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
            chainRpc.getNetworkId()
                .then(() => done.fail())
                .catch(e => {
                    expect(e.toString()).toContain("chain_getNetworkId");
                    expect(e.toString()).toContain("string");
                    expect(e.toString()).toContain("undefined");
                    done();
                });
        });
    });
});
