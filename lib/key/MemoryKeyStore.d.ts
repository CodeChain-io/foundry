import { KeyManagementAPI, KeyStore } from "./KeyStore";
/**
 * @hidden
 */
declare class KeyManager implements KeyManagementAPI {
    private privateKeyMap;
    private passphraseMap;
    private publicKeyMap;
    private mappingKeyMaker;
    constructor(keyMaker: (value: string) => string);
    getKeyList(): Promise<string[]>;
    createKey(params?: {
        passphrase?: string;
    }): Promise<string>;
    removeKey(params: {
        key: string;
    }): Promise<boolean>;
    exportRawKey(params: {
        key: string;
        passphrase?: string;
    }): Promise<string>;
    getPublicKey(params: {
        key: string;
    }): Promise<string | null>;
    sign(params: {
        key: string;
        message: string;
        passphrase?: string;
    }): Promise<string>;
}
export declare class MemoryKeyStore implements KeyStore {
    platform: KeyManager;
    asset: KeyManager;
    private getHash;
}
export {};
