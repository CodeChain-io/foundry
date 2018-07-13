import fetch from "node-fetch";

import { NodeRpc } from "./node";
import { ChainRpc } from "./chain";
import { NetworkRpc } from "./network";
import { AccountRpc } from "./account";

/**
 * @hidden
 */
const jaysonBrowserClient = require("jayson/lib/client/browser");

export class Rpc {
    private client: any;

    /**
     * RPC module for retrieving the node info.
     */
    public node: NodeRpc;
    /**
     * RPC module for accessing the blockchain.
     */
    public chain: ChainRpc;
    /**
     * RPC module for configuring P2P networking of the node.
     */
    public network: NetworkRpc;
    /**
     * RPC module for account management and signing
     */
    public account: AccountRpc;

    /**
     * @param params.server HTTP RPC server address.
     */
    constructor(params: { server: string }) {
        const { server } = params;
        this.client = jaysonBrowserClient((request: any, callback: any) => {
            fetch(server, {
                method: "POST",
                body: request,
                headers: {
                    "Content-Type": "application/json"
                }
            }).then(res => {
                return res.text();
            }).then(text => {
                return callback(null, text);
            }).catch(err => {
                return callback(err);
            });
        });

        this.node = new NodeRpc(this);
        this.chain = new ChainRpc(this);
        this.network = new NetworkRpc(this);
        this.account = new AccountRpc(this);
    }

    sendRpcRequest = (name: string, params: any[]) => {
        return new Promise<any>((resolve, reject) => {
            this.client.request(name, params, (err: any, res: any) => {
                if (err) {
                    return reject(err);
                } else if (res.error) {
                    return reject(res.error);
                }
                resolve(res.result);
            });
        });
    }
}
