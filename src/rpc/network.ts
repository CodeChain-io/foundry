import { Rpc } from ".";

export class NetworkRpc {
    private rpc: Rpc;

    /**
     * @hidden
     */
    constructor(rpc: Rpc) {
        this.rpc = rpc;
    }

    /**
     * Save secret which is used when handshaking with other node,
     * This secret may be exchanged in offline.
     * To use this saved secret, you should call 'net_connect' RPC after this RPC call.
     * @param secret Secret exchanged in offline
     * @param address Node address which RPC server will connect to using secret
     * @param port
     */
    public shareSecret(
        secret: string,
        address: string,
        port: number
    ): Promise<null> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("net_shareSecret", [secret, address, port])
                .then(result => {
                    if (result === null) {
                        return resolve(null);
                    }
                    reject(
                        Error(
                            `Expected net_shareSecret to return null but it returned ${result}`
                        )
                    );
                })
                .catch(reject);
        });
    }

    /**
     * Connect to node
     * @param address Node address which to connect
     * @param port
     */
    public connect(address: string, port: number): Promise<null> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("net_connect", [address, port])
                .then(result => {
                    if (result === null) {
                        return resolve(null);
                    }
                    reject(
                        Error(
                            `Expected net_connect to return null but it returned ${result}`
                        )
                    );
                })
                .catch(reject);
        });
    }

    /**
     * Disconnect from the node
     * @param address Node address which to disconnect
     * @param port
     */
    public disconnect(address: string, port: number): Promise<null> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("net_disconnect", [address, port])
                .then(result => {
                    if (result === null) {
                        return resolve(null);
                    }
                    reject(
                        Error(
                            `Expected net_disconnect to return null but it returned ${result}`
                        )
                    );
                })
                .catch(reject);
        });
    }

    /**
     * Check the node is connected
     * @param address Node address
     * @param port
     */
    public isConnected(address: string, port: number): Promise<boolean> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("net_isConnected", [address, port])
                .then(result => {
                    if (typeof result === "boolean") {
                        return resolve(result);
                    }
                    reject(
                        Error(
                            `Expected net_isConnected to return a boolean but it returned ${result}`
                        )
                    );
                })
                .catch(reject);
        });
    }

    /**
     * Get the number of established peers
     */
    public getPeerCount(): Promise<number> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("net_getPeerCount", [])
                .then(result => {
                    if (typeof result === "number") {
                        return resolve(result);
                    }
                    reject(
                        Error(
                            `Expected net_getPeerCount to return a number but it returned ${result}`
                        )
                    );
                })
                .catch(reject);
        });
    }

    /**
     * Get the addresses of established peers
     */
    public getPeers(): Promise<string[]> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("net_getEstablishedPeers", [])
                .then(result => {
                    if (!Array.isArray(result)) {
                        // FIXME: Check whether the strings are peer addresses.
                        resolve(result);
                    }
                    return reject(
                        Error(
                            `Expected net_getEstablishedPeers to return an array but it returned ${result}`
                        )
                    );
                })
                .catch(reject);
        });
    }

    /**
     * Add the IP to whitelist
     * @param ip Node IP
     */
    public addToWhitelist(ip: string): Promise<null> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("net_addToWhitelist", [ip])
                .then(result => {
                    if (result === null) {
                        return resolve(null);
                    }
                    reject(
                        Error(
                            `Expected net_addToWhitelist to return null but it returned ${result}`
                        )
                    );
                })
                .catch(reject);
        });
    }

    /**
     * Remove the IP from whitelist
     * @param ip Node IP
     */
    public removeFromWhitelist(ip: string): Promise<null> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("net_removeFromWhitelist", [ip])
                .then(result => {
                    if (result === null) {
                        return resolve(null);
                    }
                    reject(
                        Error(
                            `Expected net_removeFromWhitelist to return null but it returned ${result}`
                        )
                    );
                })
                .catch(reject);
        });
    }

    /**
     * Add the IP to blacklist
     * @param ip Node IP
     */
    public addToBlacklist(ip: string): Promise<null> {
        return new Promise((resolve, reject) => {
            this.rpc.sendRpcRequest("net_addToBlacklist", [ip]).then(result => {
                if (result === null) {
                    return resolve(null);
                }
                reject(
                    Error(
                        `Expected net_addToBlacklist to return null but it returned ${result}`
                    )
                );
            });
        });
    }

    /**
     * Remove the IP from blacklist
     * @param ip Node IP
     */
    public removeFromBlacklist(ip: string): Promise<null> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("net_removeFromBlacklist", [ip])
                .then(result => {
                    if (result === null) {
                        return resolve(null);
                    }
                    reject(
                        Error(
                            `Expected net_removeFromBlacklist to return null but it returned ${result}`
                        )
                    );
                })
                .catch(reject);
        });
    }

    /**
     * Enable whitelist
     */
    public enableWhitelist(): Promise<null> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("net_enableWhitelist", [])
                .then(result => {
                    if (result === null) {
                        return resolve(null);
                    }
                    reject(
                        Error(
                            `Expected net_enableWhitelist to return null but it returned ${result}`
                        )
                    );
                })
                .catch(reject);
        });
    }

    /**
     * Disable whitelist
     */
    public disableWhitelist(): Promise<null> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("net_disableWhitelist", [])
                .then(result => {
                    if (result === null) {
                        return resolve(null);
                    }
                    reject(
                        Error(
                            `Expected net_disableWhitelist to return null but it returned ${result}`
                        )
                    );
                })
                .catch(reject);
        });
    }

    /**
     * Enable blacklist
     */
    public enableBlacklist(): Promise<null> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("net_enableBlacklist", [])
                .then(result => {
                    if (result === null) {
                        return resolve(null);
                    }
                    reject(
                        Error(
                            `Expected net_enableBlacklist to return null but it returned ${result}`
                        )
                    );
                })
                .catch(reject);
        });
    }

    /**
     * Disable blacklist
     */
    public disableBlacklist(): Promise<null> {
        return new Promise((resolve, reject) => {
            this.rpc.sendRpcRequest("net_disableBlacklist", []).then(result => {
                if (result === null) {
                    return resolve(null);
                }
                reject(
                    Error(
                        `Expected net_disableBlacklist to return null but it returned ${result}`
                    )
                );
            });
        });
    }

    /**
     * Get the status of whitelist
     */
    public getWhitelist(): Promise<{ list: string[]; enabled: boolean }> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("net_getWhitelist", [])
                .then(result => {
                    if (result === null || typeof result !== "object") {
                        return reject(
                            Error(
                                `Expected net_getWhitelist to return an object but it returned ${result}`
                            )
                        );
                    }
                    const { list, enabled } = result;
                    if (Array.isArray(list) && typeof enabled === "boolean") {
                        // FIXME: Check whether the strings in the list are valid.
                        return resolve(result);
                    }
                    reject(
                        Error(
                            `Expected net_getWhitelist to return { list: string[], enabled: boolean } but it returned ${JSON.stringify(
                                result
                            )}`
                        )
                    );
                })
                .catch(reject);
        });
    }

    /**
     * Get the status of blacklist
     */
    public getBlacklist(): Promise<{ list: string[]; enabled: boolean }> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("net_getBlacklist", [])
                .then(result => {
                    if (result === null || typeof result !== "object") {
                        return reject(
                            Error(
                                `Expected net_getBlacklist to return an object but it returned ${result}`
                            )
                        );
                    }
                    const { list, enabled } = result;
                    if (Array.isArray(list) && typeof enabled === "boolean") {
                        // FIXME: Check whether the strings in the list are valid.
                        return resolve(result);
                    }
                    reject(
                        Error(
                            `Expected net_getBlacklist to return { list: string[], enabled: boolean } but it returned ${JSON.stringify(
                                result
                            )}`
                        )
                    );
                })
                .catch(reject);
        });
    }
}
