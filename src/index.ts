import { H160, SignedTransaction, H256, Transaction, U256, Action, Invoice } from "./primitives/index";

const jayson = require('jayson');

class SDK {
    private client;

    constructor(httpUrl) {
        this.client = jayson.client.http(httpUrl);
    }

    ping(): Promise<string> {
        return new Promise((resolve, reject) => {
            this.client.request("ping", [], (err, res) => {
                if (err) {
                    return reject(err);
                }
                resolve(res.result);
            });
        });
    }

    sendSignedTransaction(t: SignedTransaction): Promise<H256> {
        return new Promise((resolve, reject) => {
            const bytes = Array.from(t.rlpBytes()).map(byte => byte < 0x10 ? `0${byte.toString(16)}` : byte.toString(16)).join("");
            this.client.request("chain_sendSignedTransaction", [`0x${bytes}`], (err, res) => {
                if (err) {
                    return reject(err);
                } else if (res.error) {
                    return reject(res.error);
                }
                resolve(new H256(res.result));
            })
        });
    }

    // FIXME: timeout not implemented
    getTransactionInvoice(txhash: H256, _timeout: number): Promise<Invoice | null> {
        return new Promise((resolve, reject) => {
            this.client.request("chain_getTransactionInvoice", [`0x${txhash.value}`], (err, res) => {
                if (err) {
                    return reject(err);
                } else if (res.error) {
                    return reject(res.error);
                }
                resolve(new Invoice(res.result.outcome === "Success"));
            });;
        });
    }

    getNonce(address: H160, blockNumber?: number): Promise<U256 | null> {
        return new Promise((resolve, reject) => {
            this.client.request("chain_getNonce", [`0x${address.value}`, blockNumber || null], (err, res) => {
                if (err) {
                    return reject(err);
                } else if (res.error) {
                    return reject(res.error);
                }
                if (res.result) {
                    resolve(new U256(res.result));
                } else {
                    resolve(null);
                }
            });
        });
    }
}

module.exports = SDK;
