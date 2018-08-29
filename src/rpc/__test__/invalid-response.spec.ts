import { Rpc } from "../index";
import { NodeRpc } from "../node";
import { ChainRpc } from "../chain";
import { PlatformAddress } from "../../key/classes";

describe("Invalid response", () => {
    const rpc: Rpc = new Rpc({ server: "" });

    describe("NodeRpc", () => {
        const nodeRpc = new NodeRpc(rpc);
        test("ping", (done) => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
            nodeRpc.ping()
                .then(done.fail)
                .catch(e => {
                    expect(e).toEqual(Error("Expect ping() to return a string but undefined is given"));
                    done();
                });
        });

        test("getNodeVersion", (done) => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
            nodeRpc.getNodeVersion()
                .then(done.fail)
                .catch(e => {
                    expect(e).toEqual(Error("Expect getNodeVersion() to return a string but undefined is given"));
                    done();
                });
        });

        test("getCommitHash", (done) => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(undefined);
            nodeRpc.getCommitHash()
                .then(done.fail)
                .catch(e => {
                    expect(e).toEqual(Error("Expect getCommitHash() to return a string but undefined is given"));
                    done();
                });
        });
    });

    describe("ChainRpc", () => {
        const chainRpc = new ChainRpc(rpc, {});
        const address = PlatformAddress.fromAccountId("0x0000000000000000000000000000000000000000");

        test("getNonce", (done) => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(null);
            chainRpc.getNonce(address)
                .then(() => done.fail())
                .catch(e => {
                    expect(e.toString()).toContain("Expect getNonce() to return a value of U256, but an error occurred:");
                    done();
                });
        });

        test("getBalance", (done) => {
            rpc.sendRpcRequest = jest.fn().mockResolvedValueOnce(null);
            chainRpc.getBalance(address)
                .then(() => done.fail())
                .catch(e => {
                    expect(e.toString()).toContain("Expect getBalance() to return a value of U256, but an error occurred:");
                    done();
                });
        });
    });
});
