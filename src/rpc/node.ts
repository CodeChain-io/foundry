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
        return new Promise((resolve, reject) => {
            this.rpc.sendRpcRequest("ping", [])
                .then(res => {
                    const responseType = typeof res;
                    if (responseType === "string") {
                        return resolve(res);
                    }
                    return reject(Error(`Expected ping() to return a string but ${responseType} is given`));
                })
                .catch(reject);
        });
    }

    /**
     * Gets the version of CodeChain node.
     * @returns The version of CodeChain node (e.g. 0.1.0)
     */
    getNodeVersion(): Promise<string> {
        return new Promise((resolve, reject) => {
            this.rpc.sendRpcRequest("version", [])
                .then(res => {
                    const responseType = typeof res;
                    if (responseType === "string") {
                        return resolve(res);
                    }
                    return reject(Error(`Expected getNodeVersion() to return a string but ${responseType} is given`));
                })
                .catch(reject);
        });
    }

    /**
     * Gets the commit hash of the repository upon which the CodeChain executable was built.
     * @hidden
     */
    getCommitHash(): Promise<string> {
        return new Promise((resolve, reject) => {
            this.rpc.sendRpcRequest("commitHash", [])
                .then(res => {
                    const responseType = typeof res;
                    if (responseType === "string") {
                        return resolve(res);
                    }
                    return reject(Error(`Expected getCommitHash() to return a string but ${responseType} is given`));
                })
                .catch(reject);
        });
    }
}
