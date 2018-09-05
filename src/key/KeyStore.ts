export interface KeyManagementAPI {
    getKeyList(): Promise<string[]>;
    createKey(params?: { passphrase?: string }): Promise<string>;
    removeKey(params: { key: string; passphrase?: string }): Promise<boolean>;
    getPublicKey(params: { key: string }): Promise<string | null>;
    sign(params: {
        key: string;
        message: string;
        passphrase?: string;
    }): Promise<string>;
}

export interface KeyStore {
    platform: KeyManagementAPI;
    asset: KeyManagementAPI;
}
