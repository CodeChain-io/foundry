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
    shareSecret(secret: string, address: string, port: number): Promise<null> {
        return this.rpc.sendRpcRequest(
            "net_shareSecret",
            [secret, address, port]
        );
    }

    /**
     * Connect to node
     * @param address Node address which to connect
     * @param port
     */
    connect(address: string, port: number): Promise<null> {
        return this.rpc.sendRpcRequest(
            "net_connect",
            [address, port]
        );
    }

    /**
    * Disconnect from the node
    * @param address Node address which to disconnect
    * @param port
    */
    disconnect(address: string, port: number): Promise<null> {
        return this.rpc.sendRpcRequest(
            "net_disconnect",
            [address, port]
        );
    }

    /**
    * Check the node is connected
    * @param address Node address
    * @param port
    */
    isConnected(address: string, port: number): Promise<boolean> {
        return this.rpc.sendRpcRequest(
            "net_isConnected",
            [address, port]
        );
    }
}
