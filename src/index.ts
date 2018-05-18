import { H160, SignedParcel, H256, Parcel, U256, Invoice } from "./primitives/index";

const jayson = require("jayson");

export class SDK {
    private client: any;

    constructor(httpUrl: string) {
        this.client = jayson.client.http(httpUrl);
    }

    ping(): Promise<string> {
        return new Promise((resolve, reject) => {
            this.client.request("ping", [], (err: any, res: any) => {
                if (err) {
                    return reject(err);
                }
                resolve(res.result);
            });
        });
    }

    sendSignedParcel(t: SignedParcel): Promise<H256> {
        return new Promise((resolve, reject) => {
            const bytes = Array.from(t.rlpBytes()).map(byte => byte < 0x10 ? `0${byte.toString(16)}` : byte.toString(16)).join("");
            this.client.request("chain_sendSignedParcel", [`0x${bytes}`], (err: any, res: any) => {
                if (err) {
                    return reject(err);
                } else if (res.error) {
                    return reject(res.error);
                }
                resolve(new H256(res.result));
            });
        });
    }

    // FIXME: will be replaced with getParcelInvoices
    // FIXME: timeout not implemented
    getParcelInvoice(txhash: H256, _timeout: number): Promise<Invoice | null> {
        return new Promise((resolve, reject) => {
            this.client.request("chain_getParcelInvoice", [`0x${txhash.value}`], (err: any, res: any) => {
                if (err) {
                    return reject(err);
                } else if (res.error) {
                    return reject(res.error);
                }
                resolve(new Invoice(res.result.outcome === "Success"));
            });
        });
    }

    getBalance(address: H160, blockNumber?: number): Promise<U256 | null> {
        return new Promise((resolve, reject) => {
            this.client.request("chain_getBalance", [`0x${address.value}`, blockNumber || null], (err: any, res: any) => {
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

    getNonce(address: H160, blockNumber?: number): Promise<U256 | null> {
        return new Promise((resolve, reject) => {
            this.client.request("chain_getNonce", [`0x${address.value}`, blockNumber || null], (err: any, res: any) => {
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

    getBlockNumber(): Promise<number> {
        return new Promise((resolve, reject) => {
            this.client.request("chain_getBlockNumber", [], (err: any, res: any) => {
                if (err) {
                    return reject(err);
                } else if (res.error) {
                    return reject(res.error);
                }
                resolve(res.result);
            });
        });
    }

    getBlockHash(blockNumber: number): Promise<H256 | null> {
        return new Promise((resolve, reject) => {
            this.client.request("chain_getBlockHash", [blockNumber], (err: any, res: any) => {
                if (err) {
                    return reject(err);
                } else if (res.error) {
                    return reject(res.error);
                }
                resolve(res.result ? new H256(res.result) : null);
            });
        });
    }

    getBlock(hash: H256): Promise<any | null> {
        return new Promise((resolve, reject) => {
            this.client.request("chain_getBlockByHash", [`0x${hash.value}`], (err: any, res: any) => {
                if (err) {
                    return reject(err);
                } else if (res.error) {
                    return reject(res.error);
                }
                // FIXME: introduce Block primitive
                resolve(res.result);
            });
        });
    }
}
