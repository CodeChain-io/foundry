import { NodeRpc } from "./node";

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
