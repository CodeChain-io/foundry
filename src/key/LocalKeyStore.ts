import { CCKey } from "codechain-keystore";
import { KeyStore } from "./KeyStore";

export class LocalKeyStore implements KeyStore {
    public static async create(): Promise<KeyStore> {
        const cckey = await CCKey.create({});
        return new LocalKeyStore(cckey);
    }

    public static async createForTest(): Promise<KeyStore> {
        const cckey = await CCKey.create({ useMemoryDB: true });
        return new LocalKeyStore(cckey);
    }
    public cckey: CCKey;

    public platform = {
        getKeyList: (): Promise<string[]> => {
            return this.cckey.platform.getKeys();
        },

        createKey: (params: { passphrase?: string } = {}): Promise<string> => {
            return this.cckey.platform.createKey(params);
        },

        removeKey: (params: { publicKey: string }): Promise<boolean> => {
            const { publicKey } = params;
            return this.cckey.platform.deleteKey({ publicKey });
        },

        sign: (params: {
            publicKey: string;
            message: string;
            passphrase?: string;
        }): Promise<string> => {
            const { passphrase = "" } = params;
            return this.cckey.platform.sign({ ...params, passphrase });
        }
    };

    public asset = {
        getKeyList: (): Promise<string[]> => {
            return this.cckey.asset.getKeys();
        },

        createKey: (params: { passphrase?: string } = {}): Promise<string> => {
            return this.cckey.asset.createKey(params);
        },

        removeKey: (params: { publicKey: string }): Promise<boolean> => {
            const { publicKey } = params;
            return this.cckey.asset.deleteKey({ publicKey });
        },

        sign: (params: {
            publicKey: string;
            message: string;
            passphrase?: string;
        }): Promise<string> => {
            const { passphrase = "" } = params;
            return this.cckey.asset.sign({ ...params, passphrase });
        }
    };

    public mapping = {
        add: (params: { key: string; value: string }): Promise<void> => {
            return this.cckey.mapping.add(params);
        },

        get: async (params: { key: string }): Promise<string> => {
            const pk = await this.cckey.mapping.get(params);
            return pk as string;
        }
    };

    private constructor(cckey: CCKey) {
        this.cckey = cckey;
    }

    public close() {
        return this.cckey.close();
    }
}
