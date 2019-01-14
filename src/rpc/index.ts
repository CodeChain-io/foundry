import fetch from "node-fetch";

import { AccountRpc } from "./account";
import { ChainRpc } from "./chain";
import { DevelRpc } from "./devel";
import { EngineRpc } from "./engine";
import { NetworkRpc } from "./network";
import { NodeRpc } from "./node";

/**
 * @hidden
 */
const jaysonBrowserClient = require("jayson/lib/client/browser");

export class Rpc {
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
     * RPC module for retrieving the engine info.
     */
    public engine: EngineRpc;
    /**
     * RPC module for developer functions
     */
    public devel: DevelRpc;
    private client: any;

    /**
     * @param params.server HTTP RPC server address.
     * @param params.options.transactionSigner The default account to sign the tx
     * @param params.options.transactionFee The default quantity for the tx fee
     */
    constructor(params: {
        server: string;
        options?: {
            transactionSigner?: string;
            transactionFee?: number;
        };
    }) {
        const { server, options = {} } = params;
        this.client = jaysonBrowserClient((request: any, callback: any) => {
            fetch(server, {
                method: "POST",
                body: request,
                headers: {
                    "Content-Type": "application/json"
                }
            })
                .then(res => {
                    return res.text();
                })
                .then(text => {
                    return callback(null, text);
                })
                .catch(err => {
                    return callback(err);
                });
        });

        this.node = new NodeRpc(this);
        this.chain = new ChainRpc(this, options);
        this.network = new NetworkRpc(this);
        this.account = new AccountRpc(this, options);
        this.engine = new EngineRpc(this);
        this.devel = new DevelRpc(this);
    }

    public sendRpcRequest = (name: string, params: any[]) => {
        return new Promise<any>((resolve, reject) => {
            this.client.request(name, params, (err: any, res: any) => {
                if (err) {
                    return reject(
                        Error(`An error occurred while ${name}: ${err}`)
                    );
                } else if (res.error) {
                    // FIXME: Throw Error with a description
                    return reject(res.error);
                }
                resolve(res.result);
            });
        });
    };
}
