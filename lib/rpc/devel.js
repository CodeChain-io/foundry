"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
class DevelRpc {
    /**
     * @hidden
     */
    constructor(rpc) {
        this.rpc = rpc;
    }
    /**
     * Starts and Enable sealing parcels.
     * @returns null
     */
    startSealing() {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("devel_startSealing", [])
                .then(result => {
                if (result === null) {
                    return resolve(null);
                }
                reject(Error(`Expected devel_startSealing to return null but it returned ${result}`));
            })
                .catch(reject);
        });
    }
    /**
     * Stops and Disable sealing parcels.
     * @returns null
     */
    stopSealing() {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("devel_stopSealing", [])
                .then(result => {
                if (result === null) {
                    return resolve(null);
                }
                reject(Error(`Expected devel_stopSealing to return null but it returned ${result}`));
            })
                .catch(reject);
        });
    }
}
exports.DevelRpc = DevelRpc;
