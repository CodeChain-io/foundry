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
    startSealing(): Promise<null> {
        return this.rpc.sendRpcRequest("devel_startSealing", []);
    }

    /**
     * Stops and Disable sealing parcels.
     * @returns null
     */
    stopSealing(): Promise<null> {
        return this.rpc.sendRpcRequest("devel_stopSealing", []);
    }
}
