
export interface KeyStore {
    getKeyList(): Promise<string[]>;
    createKey(params?: { passphrase?: string }): Promise<string>;
    removeKey(params: { publicKey: string, passphrase?: string }): Promise<boolean>;
    sign(params: { publicKey: string, message: string, passphrase?: string }): Promise<string>;

    addPKH(params: { publicKey: string }): Promise<string>;
    getPK(params: { hash: string }): Promise<string>;
}
