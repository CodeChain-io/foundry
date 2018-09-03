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
