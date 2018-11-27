import { H256 } from "codechain-primitives";

import { Rpc } from ".";

export class DevelRpc {
    private rpc: Rpc;

    /**
     * @hidden
     */
    constructor(rpc: Rpc) {
        this.rpc = rpc;
    }

    /**
     * Gets keys of the state trie with the given offset and limit.
     * @param offset number
     * @param limit number
     * @returns H256[]
     */
    public getStateTrieKeys(offset: number, limit: number): Promise<H256[]> {
        if (
            typeof offset !== "number" ||
            !Number.isInteger(offset) ||
            offset < 0
        ) {
            throw Error(
                `Expected the first argument to be non-negative integer but found ${offset}`
            );
        }
        if (
            typeof limit !== "number" ||
            !Number.isInteger(limit) ||
            limit <= 0
        ) {
            throw Error(
                `Expected the second argument to be posivit integer but found ${limit}`
            );
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("devel_getStateTrieKeys", [offset, limit])
                .then(result => {
                    if (!Array.isArray(result)) {
                        return reject(
                            Error(
                                `Expected devel_getStateTrieKeys to return an array but it returned ${result}`
                            )
                        );
                    }
                    result.forEach((value, index, arr) => {
                        try {
                            arr[index] = new H256(value);
                        } catch (e) {
                            return reject(
                                Error(
                                    `Expected devel_getStateTrieKeys() to return an array of H256, but an error occurred: ${e.toString()}`
                                )
                            );
                        }
                    });
                    resolve(result);
                })
                .catch(reject);
        });
    }

    /**
     * Gets the value of the state trie with the given key.
     * @param key H256
     * @returns string[]
     */
    public getStateTrieValue(key: H256): Promise<string[]> {
        if (!H256.check(key)) {
            throw Error(
                `Expected the first argument to be an H256 value but found ${key}`
            );
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("devel_getStateTrieValue", [
                    `0x${H256.ensure(key).value}`
                ])
                .then(result => {
                    if (!Array.isArray(result)) {
                        return reject(
                            Error(
                                `Expected devel_getStateTrieValue to return an array but it returned ${result}`
                            )
                        );
                    }
                    result.forEach((value, index) => {
                        if (typeof value !== "string") {
                            return reject(
                                Error(
                                    `Expected devel_getStateTrieValue to return an array of strings but found ${value} at ${index}`
                                )
                            );
                        }
                    });
                    resolve(result);
                })
                .catch(reject);
        });
    }

    /**
     * Starts and Enable sealing parcels.
     * @returns null
     */
    public startSealing(): Promise<null> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("devel_startSealing", [])
                .then(result => {
                    if (result === null) {
                        return resolve(null);
                    }
                    reject(
                        Error(
                            `Expected devel_startSealing to return null but it returned ${result}`
                        )
                    );
                })
                .catch(reject);
        });
    }

    /**
     * Stops and Disable sealing parcels.
     * @returns null
     */
    public stopSealing(): Promise<null> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("devel_stopSealing", [])
                .then(result => {
                    if (result === null) {
                        return resolve(null);
                    }
                    reject(
                        Error(
                            `Expected devel_stopSealing to return null but it returned ${result}`
                        )
                    );
                })
                .catch(reject);
        });
    }
}
