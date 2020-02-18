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
    private server: string;

    /**
     * @param params.server HTTP RPC server address.
     * @param params.options.transactionSigner The default account to sign the tx
     */
    constructor(params: {
        server: string;
        options?: {
            transactionSigner?: string;
            fallbackServers?: string[];
        };
    }) {
        const { server, options = {} } = params;
        this.server = server;
        this.client = (rpcServer: string) => {
            return jaysonBrowserClient((request: any, callback: any) => {
                fetch(rpcServer, {
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
        };

        this.node = new NodeRpc(this);
        this.chain = new ChainRpc(this, options);
        this.network = new NetworkRpc(this);
        this.account = new AccountRpc(this);
        this.engine = new EngineRpc(this, options);
        this.devel = new DevelRpc(this);
    }

    public sendRpcRequest = async (
        name: string,
        params: any[],
        options?: { id?: string; fallbackServers?: string[] }
    ) => {
        const { fallbackServers } = options || { fallbackServers: [] };
        const allServers: string[] =
            fallbackServers === undefined
                ? [this.server]
                : [this.server, ...fallbackServers];
        const errors: any[] = [];
        for (const server of allServers) {
            const { id } = options || { id: undefined };
            try {
                return new Promise<any>((resolve, reject) => {
                    this.client(server).request(
                        name,
                        params,
                        id,
                        (err: any, res: any) => {
                            if (err || res.error) {
                                reject(err || res.error);
                            } else {
                                resolve(res.result);
                            }
                        }
                    );
                });
            } catch (err) {
                errors.push({ [server]: err });
            }
        }
        if (errors.length === 1) {
            return Promise.reject(errors[0][this.server]);
        }
        return Promise.reject(errors);
    };
}
