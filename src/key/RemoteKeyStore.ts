import * as rp from "request-promise";

import { KeyManagementAPI, KeyStore } from "./KeyStore";

type KeyType = "asset" | "platform";

class RemoteKeyManager implements KeyManagementAPI {
    public keystoreURL: string;
    public keyType: string;

    constructor(keystoreURL: string, keyType: KeyType) {
        this.keystoreURL = keystoreURL;
        this.keyType = keyType;
    }

    public async getKeyList(): Promise<string[]> {
        const response = await rp.get(`${this.keystoreURL}/api/keys`, {
            body: { keyType: this.keyType },
            json: true
        });

        if (!response.success) {
            throw Error(response.error);
        }

        return response.result;
    }

    public async createKey(
        params: { passphrase?: string } = {}
    ): Promise<string> {
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

    public async removeKey(params: { key: string }): Promise<boolean> {
        const response = await rp.delete(
            `${this.keystoreURL}/api/keys/${params.key}`,
            {
                body: {
                    keyType: this.keyType
                },
                json: true
            }
        );

        if (!response.success) {
            throw Error(response.error);
        }

        return response.result;
    }

    public async exportRawKey(params: {
        key: string;
        passphrase?: string;
    }): Promise<string> {
        const response = await rp.post(
            `${this.keystoreURL}/api/keys/${params.key}/sign`,
            {
                body: {
                    keyType: this.keyType,
                    passphrase: params.passphrase
                },
                json: true
            }
        );

        if (!response.success) {
            throw Error(response.error);
        }

        return response.result;
    }

    public async getPublicKey(params: { key: string }): Promise<string | null> {
        const response = await rp.get(
            `${this.keystoreURL}/api/keys/${params.key}/publicKey`,
            {
                body: {
                    keyType: this.keyType
                },
                json: true
            }
        );

        if (!response.success) {
            throw Error(response.error);
        }

        return response.result;
    }

    public async sign(params: {
        key: string;
        message: string;
        passphrase?: string;
    }): Promise<string> {
        const response = await rp.post(
            `${this.keystoreURL}/api/keys/${params.key}/sign`,
            {
                body: {
                    keyType: this.keyType,
                    passphrase: params.passphrase,
                    publicKey: params.key,
                    message: params.message
                },
                json: true
            }
        );

        if (!response.success) {
            throw Error(response.error);
        }

        return response.result;
    }
}

export class RemoteKeyStore implements KeyStore {
    public static async create(keystoreURL: string): Promise<KeyStore> {
        const keystore = new RemoteKeyStore(keystoreURL);
        await keystore.ping();
        return keystore;
    }

    // Use only this method for test purpose
    public static createUnsafe(keystoreURL: string): KeyStore {
        const keystore = new RemoteKeyStore(keystoreURL);
        keystore.ping();
        return keystore;
    }
    public platform: KeyManagementAPI;
    public asset: KeyManagementAPI;
    public keystoreURL: string;

    public mapping = {
        get: async (params: { key: string }): Promise<string> => {
            const response = await rp.get(
                `${this.keystoreURL}/api/mapping/${params.key}`,
                {
                    json: true
                }
            );

            if (!response.success) {
                throw Error(response.error);
            }

            return response.result;
        }
    };

    public constructor(keystoreURL: string) {
        this.keystoreURL = keystoreURL;
        this.platform = new RemoteKeyManager(keystoreURL, "platform");
        this.asset = new RemoteKeyManager(keystoreURL, "asset");
    }

    public async ping(): Promise<void> {
        const response = await rp.get(`${this.keystoreURL}/ping`, {
            json: true
        });

        if (!response.success) {
            throw Error(response.error);
        }

        return response.result;
    }
}
