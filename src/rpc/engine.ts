import { PlatformAddress, U64 } from "codechain-primitives";

import { Rpc } from ".";

export class EngineRpc {
    private rpc: Rpc;

    /**
     * @hidden
     */
    constructor(rpc: Rpc) {
        this.rpc = rpc;
    }

    /**
     * Gets coinbase's account id.
     * @returns PlatformAddress or null
     */
    public getCoinbase(): Promise<PlatformAddress | null> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("engine_getCoinbase", [])
                .then(result => {
                    try {
                        resolve(
                            result === null
                                ? null
                                : PlatformAddress.fromString(result)
                        );
                    } catch (e) {
                        reject(
                            Error(
                                `Expected engine_getCoinbase to return a PlatformAddress string or null, but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Gets coinbase's account id.
     * @returns PlatformAddress or null
     */
    public getBlockReward(): Promise<U64> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("engine_getBlockReward", [])
                .then(result => {
                    try {
                        resolve(U64.ensure(result));
                    } catch (e) {
                        reject(
                            Error(
                                `Expected engine_getBlockReward to return a U64, but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Gets coinbase's account id.
     * @returns PlatformAddress or null
     */
    public getRecommendedConfirmation(): Promise<number> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("engine_getRecommendedConfirmation", [])
                .then(result => {
                    if (typeof result === "number") {
                        return resolve(result);
                    }
                    reject(
                        Error(
                            `Expected engine_getRecommendedConfirmation to return a number but it returned ${result}`
                        )
                    );
                })
                .catch(reject);
        });
    }
}
