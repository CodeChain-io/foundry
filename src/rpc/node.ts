import { Rpc } from ".";

export class NodeRpc {
    private rpc: Rpc;

    /**
     * @hidden
     */
    constructor(rpc: Rpc) {
        this.rpc = rpc;
    }

    /**
     * Sends ping to check whether CodeChain's RPC server is responding or not.
     * @returns String "pong"
     */
    ping(): Promise<string> {
        return this.rpc.sendRpcRequest("ping", []);
    }

    /**
     * Gets the version of CodeChain node.
     * @returns The version of CodeChain node (e.g. 0.1.0)
     */
    getNodeVersion(): Promise<string> {
        return this.rpc.sendRpcRequest("version", []);
    }

    /**
     * Gets the commit hash of the repository upon which the CodeChain executable was built.
     * @hidden
     */
    getCommitHash(): Promise<string> {
        return this.rpc.sendRpcRequest("commitHash", []);
    }
}
