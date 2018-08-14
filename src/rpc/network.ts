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

    /**
     * Add the IP to whitelist
     * @param ip Node IP
     */
    addToWhiteList(ip: string): Promise<null> {
        return this.rpc.sendRpcRequest(
            "net_addToWhitelist",
            [ip]
        );
    }

    /**
     * Remove the IP from whitelist
     * @param ip Node IP
     */
    removeFromWhiteList(ip: string): Promise<null> {
        return this.rpc.sendRpcRequest(
            "net_removeFromWhitelist",
            [ip]
        );
    }

    /**
     * Add the IP to blacklist
     * @param ip Node IP
     */
    addToBlacklist(ip: string): Promise<null> {
        return this.rpc.sendRpcRequest(
            "net_addToBlacklist",
            [ip]
        );
    }

    /**
     * Remove the IP from blacklist
     * @param ip Node IP
     */
    removeFromBlackList(ip: string): Promise<null> {
        return this.rpc.sendRpcRequest(
            "net_removeFromBlacklist",
            [ip]
        );
    }

    /**
     * Enable whitelist
     */
    enableWhiteList(): Promise<null> {
        return this.rpc.sendRpcRequest(
            "net_enableWhitelist",
            []
        );
    }

    /**
     * Disable whitelist
     */
    disableWhiteList(): Promise<null> {
        return this.rpc.sendRpcRequest(
            "net_disableWhitelist",
            []
        );
    }

    /**
     * Enable blacklist
     */
    enableBlackList(): Promise<null> {
        return this.rpc.sendRpcRequest(
            "net_enableBlacklist",
            []
        );
    }

    /**
     * Disable blacklist
     */
    disableBlackList(): Promise<null> {
        return this.rpc.sendRpcRequest(
            "net_disableBlacklist",
            []
        );
    }

    /**
     * Get the status of whitelist
     */
    getWhitelist(): Promise<{ list: string[], enabled: boolean }> {
        return this.rpc.sendRpcRequest(
            "net_getWhitelist",
            []
        );
    }

    /**
     * Get the status of blacklist
     */
    getBlacklist(): Promise<{ list: string[], enabled: boolean }> {
        return this.rpc.sendRpcRequest(
            "net_getBlacklist",
            []
        );
    }
}
