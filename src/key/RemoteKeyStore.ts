import * as _ from "lodash";
import * as rp from "request-promise";

import { KeyStore, KeyManagementAPI } from "./KeyStore";

type KeyType = "asset" | "platform";

class RemoteKeymanager implements KeyManagementAPI {
    keystoreURL: string;
    keyType: string;

    constructor(keystoreURL: string, keyType: KeyType) {
        this.keystoreURL = keystoreURL;
        this.keyType = keyType;
    }

    async getKeyList(): Promise<string[]> {
        const response = await rp.get(`${this.keystoreURL}/api/keys`, {
            body: { keyType: this.keyType },
            json: true
        });

        if (!response.success) {
            throw Error(response.error);
        }

        return response.result;
    }

    async createKey(params: { passphrase?: string } = {}): Promise<string> {
        const response = await rp.post(`${this.keystoreURL}/api/keys`, {
            body: {
                keyType: this.keyType,
                passphrase: params.passphrase
            },
            json: true
        });

        if (!response.success) {
            throw Error(response.error);
        }

        return response.result;
    }

    async removeKey(params: { publicKey: string, passphrase?: string }): Promise<boolean> {
        const response = await rp.post(`${this.keystoreURL}/api/keys/${params.publicKey}/remove`, {
            body: {
                keyType: this.keyType,
                passphrase: params.passphrase
            },
            json: true
        });

        if (!response.success) {
            throw Error(response.error);
        }

        return response.result;
    }

    async sign(params: { publicKey: string, message: string, passphrase?: string }): Promise<string> {
        const response = await rp.post(`${this.keystoreURL}/api/keys/${params.publicKey}/sign`, {
            body: {
                keyType: this.keyType,
                passphrase: params.passphrase,
                publicKey: params.publicKey,
                message: params.message
            },
            json: true
        });

        if (!response.success) {
            throw Error(response.error);
        }

        return response.result;
    }
}

export class RemoteKeyStore implements KeyStore {
    platform: KeyManagementAPI;
    asset: KeyManagementAPI;
    keystoreURL: string;

    private constructor(keystoreURL: string) {
        this.keystoreURL = keystoreURL;
        this.platform = new RemoteKeymanager(keystoreURL, "platform");
        this.asset = new RemoteKeymanager(keystoreURL, "asset");
    }

    static async create(keystoreURL: string): Promise<KeyStore> {
        const keystore = new RemoteKeyStore(keystoreURL);
        await keystore.ping();
        return keystore;
    }

    // Use only this method for test purpose
    static createUnsafe(keystoreURL: string): KeyStore {
        const keystore = new RemoteKeyStore(keystoreURL);
        keystore.ping();
        return keystore;
    }

    pkh = {
        addPKH: async (params: { publicKey: string; }): Promise<string> => {
            const response = await rp.post(`${this.keystoreURL}/api/pkhs`, {
                body: {
                    publicKey: params.publicKey
                },
                json: true
            });

            if (!response.success) {
                throw Error(response.error);
            }

            return response.result;
        },

        getPK: async (params: { hash: string; }): Promise<string> => {
            const response = await rp.get(`${this.keystoreURL}/api/pkhs/${params.hash}`, {
                json: true
            });

            if (!response.success) {
                throw Error(response.error);
            }

            return response.result;
        }
    };

    async ping(): Promise<void> {
        const response = await rp.get(`${this.keystoreURL}/ping`, {
            json: true
        });

        if (!response.success) {
            throw Error(response.error);
        }

        return response.result;
    }
}
