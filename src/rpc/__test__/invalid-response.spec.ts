import { Rpc } from "../index";
import { NodeRpc } from "../node";

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
});
