"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
class NodeRpc {
    /**
     * @hidden
     */
    constructor(rpc) {
        this.rpc = rpc;
    }
    /**
     * Sends ping to check whether CodeChain's RPC server is responding or not.
     * @returns String "pong"
     */
    ping() {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("ping", [])
                .then(result => {
                if (typeof result === "string") {
                    return resolve(result);
                }
                return reject(Error(`Expected ping() to return a string but it returned ${result}`));
            })
                .catch(reject);
        });
    }
    /**
     * Gets the version of CodeChain node.
     * @returns The version of CodeChain node (e.g. 0.1.0)
     */
    getNodeVersion() {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("version", [])
                .then(result => {
                if (typeof result === "string") {
                    return resolve(result);
                }
                return reject(Error(`Expected getNodeVersion() to return a string but it returned ${result}`));
            })
                .catch(reject);
        });
    }
    /**
     * Gets the commit hash of the repository upon which the CodeChain executable was built.
     * @hidden
     */
    getCommitHash() {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("commitHash", [])
                .then(result => {
                if (typeof result === "string") {
                    return resolve(result);
                }
                return reject(Error(`Expected getCommitHash() to return a string but it returned ${result}`));
            })
                .catch(reject);
        });
    }
}
exports.NodeRpc = NodeRpc;
