export interface KeyManagementAPI {
    getKeyList(): Promise<string[]>;
    createKey(params?: { passphrase?: string }): Promise<string>;
    removeKey(params: { key: string }): Promise<boolean>;
    exportRawKey(params: { key: string; passphrase?: string }): Promise<string>;
    getPublicKey(params: {
        key: string;
        passphrase?: string;
    }): Promise<string | null>;
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
