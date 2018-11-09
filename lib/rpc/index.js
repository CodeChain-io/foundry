"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const node_fetch_1 = require("node-fetch");
const account_1 = require("./account");
const chain_1 = require("./chain");
const devel_1 = require("./devel");
const network_1 = require("./network");
const node_1 = require("./node");
/**
 * @hidden
 */
const jaysonBrowserClient = require("jayson/lib/client/browser");
class Rpc {
    /**
     * @param params.server HTTP RPC server address.
     * @param params.options.parcelSigner The default account to sign the parcel
     * @param params.options.parcelFee The default amount for the parcel fee
     */
    constructor(params) {
        this.sendRpcRequest = (name, params) => {
            return new Promise((resolve, reject) => {
                this.client.request(name, params, (err, res) => {
                    if (err) {
                        return reject(Error(`An error occurred while ${name}: ${err}`));
                    }
                    else if (res.error) {
                        // FIXME: Throw Error with a description
                        return reject(res.error);
                    }
                    resolve(res.result);
                });
            });
        };
        const { server, options = {} } = params;
        this.client = jaysonBrowserClient((request, callback) => {
            node_fetch_1.default(server, {
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
        this.node = new node_1.NodeRpc(this);
        this.chain = new chain_1.ChainRpc(this, options);
        this.network = new network_1.NetworkRpc(this);
        this.account = new account_1.AccountRpc(this, options);
        this.devel = new devel_1.DevelRpc(this);
    }
}
exports.Rpc = Rpc;
