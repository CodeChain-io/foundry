import * as _ from "lodash";

import { KeyStore } from "./KeyStore";
import { CCKey } from "codechain-keystore";

export class LocalKeyStore implements KeyStore {
    cckey: CCKey;
    private constructor(cckey: CCKey) {
        this.cckey = cckey;
    }

    static async create(): Promise<KeyStore> {
        const cckey = await CCKey.create({});
        return new LocalKeyStore(cckey);
    }

    static async createForTest(): Promise<KeyStore> {
        const cckey = await CCKey.create({ useMemoryDB: true });
        return new LocalKeyStore(cckey);
    }

    getKeyList(): Promise<string[]> {
        return this.cckey.getKeys();
    }

    createKey(params: { passphrase?: string } = {}): Promise<string> {
        return this.cckey.createKey(params);
    }

    removeKey(params: { publicKey: string, passphrase?: string }): Promise<boolean> {
        const { publicKey, passphrase = "" } = params;
        return this.cckey.deleteKey({ publicKey, passphrase });
    }

    sign(params: { publicKey: string, message: string, passphrase?: string }): Promise<string> {
        const { passphrase = "" } = params;
        return this.cckey.signKey({ ...params, passphrase });
    }

    addPKH(params: { publicKey: string; }): Promise<string> {
        return this.cckey.insertPKH(params);
    }

    getPK(params: { hash: string; }): Promise<string> {
        return this.getPK(params);
    }

    close(): Promise<void> {
        return this.cckey.close();
    }
}
