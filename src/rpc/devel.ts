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
        return this.rpc.sendRpcRequest("devel_startSealing", []);
    }

    /**
     * Stops and Disable sealing parcels.
     * @returns null
     */
    public stopSealing(): Promise<null> {
        return this.rpc.sendRpcRequest("devel_stopSealing", []);
    }
}
