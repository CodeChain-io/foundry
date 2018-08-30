export interface KeyManagementAPI {
    getKeyList(): Promise<string[]>;
    createKey(params?: { passphrase?: string }): Promise<string>;
    removeKey(params: {
        publicKey: string;
        passphrase?: string;
    }): Promise<boolean>;
    sign(params: {
        publicKey: string;
        message: string;
        passphrase?: string;
    }): Promise<string>;
}

export interface KeyStore {
    platform: KeyManagementAPI;
    asset: KeyManagementAPI;

    mapping: {
        add(params: { key: string; value: string }): Promise<void>;
        get(params: { key: string }): Promise<string>;
    };
}
