import { Rpc } from ".";
export declare class NodeRpc {
    private rpc;
    /**
     * @hidden
     */
    constructor(rpc: Rpc);
    /**
     * Sends ping to check whether CodeChain's RPC server is responding or not.
     * @returns String "pong"
     */
    ping(): Promise<string>;
    /**
     * Gets the version of CodeChain node.
     * @returns The version of CodeChain node (e.g. 0.1.0)
     */
    getNodeVersion(): Promise<string>;
    /**
     * Gets the commit hash of the repository upon which the CodeChain executable was built.
     * @hidden
     */
    getCommitHash(): Promise<string>;
}
