import { Rpc } from ".";
export declare class DevelRpc {
    private rpc;
    /**
     * @hidden
     */
    constructor(rpc: Rpc);
    /**
     * Starts and Enable sealing parcels.
     * @returns null
     */
    startSealing(): Promise<null>;
    /**
     * Stops and Disable sealing parcels.
     * @returns null
     */
    stopSealing(): Promise<null>;
}
