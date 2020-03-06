import { CCKey } from "foundry-keystore";
import * as _ from "lodash";
import { KeyStore } from "./KeyStore";

export class LocalKeyStore implements KeyStore {
    public static async create(
        options: { dbPath?: string } = {}
    ): Promise<KeyStore> {
        const cckey = await CCKey.create(options);
        return new LocalKeyStore(cckey);
    }

    public static async createForTest(): Promise<KeyStore> {
        const cckey = await CCKey.create({ dbType: "in-memory" });
        return new LocalKeyStore(cckey);
    }
    public cckey: CCKey;

    public constructor(cckey: CCKey) {
        this.cckey = cckey;
    }

    public getKeyList = async (): Promise<string[]> => {
        return await this.cckey.keystore.getKeys();
    };

    public createKey = (
        params: { passphrase?: string } = {}
    ): Promise<string> => {
        return this.cckey.keystore.createKey(params);
    };

    public removeKey = (params: { key: string }): Promise<boolean> => {
        const { key } = params;
        return this.cckey.keystore.deleteKey({ key });
    };

    public exportRawKey = (params: {
        key: string;
        passphrase?: string;
    }): Promise<string> => {
        const { passphrase = "" } = params;
        return this.cckey.keystore.exportRawKey({ ...params, passphrase });
    };

    public getPublicKey = (params: {
        key: string;
        passphrase?: string;
    }): Promise<string | null> => {
        const { key, passphrase = "" } = params;
        return this.cckey.keystore.getPublicKey({ key, passphrase });
    };

    public sign = (params: {
        key: string;
        message: string;
        passphrase?: string;
    }): Promise<string> => {
        const { passphrase = "" } = params;
        return this.cckey.keystore.sign({ ...params, passphrase });
    };

    public close() {
        return this.cckey.close();
    }
}
