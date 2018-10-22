import { CCKey } from "codechain-keystore";
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

    public platform = {
        getKeyList: (): Promise<string[]> => {
            return this.cckey.platform.getKeys();
        },

        createKey: (params: { passphrase?: string } = {}): Promise<string> => {
            return this.cckey.platform.createKey(params);
        },

        removeKey: (params: { key: string }): Promise<boolean> => {
            const { key } = params;
            return this.cckey.platform.deleteKey({ key });
        },

        exportRawKey: (params: {
            key: string;
            passphrase?: string;
        }): Promise<string> => {
            const { passphrase = "" } = params;
            return this.cckey.platform.exportRawKey({ ...params, passphrase });
        },

        getPublicKey: (params: {
            key: string;
            passphrase?: string;
        }): Promise<string | null> => {
            const { key, passphrase = "" } = params;
            return this.cckey.platform.getPublicKey({ key, passphrase });
        },

        sign: (params: {
            key: string;
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

        removeKey: (params: { key: string }): Promise<boolean> => {
            const { key } = params;
            return this.cckey.asset.deleteKey({ key });
        },

        exportRawKey: (params: {
            key: string;
            passphrase?: string;
        }): Promise<string> => {
            const { passphrase = "" } = params;
            return this.cckey.asset.exportRawKey({ ...params, passphrase });
        },

        getPublicKey: (params: {
            key: string;
            passphrase?: string;
        }): Promise<string | null> => {
            const { key, passphrase = "" } = params;
            return this.cckey.asset.getPublicKey({ key, passphrase });
        },

        sign: (params: {
            key: string;
            message: string;
            passphrase?: string;
        }): Promise<string> => {
            const { passphrase = "" } = params;
            return this.cckey.asset.sign({ ...params, passphrase });
        }
    };

    public constructor(cckey: CCKey) {
        this.cckey = cckey;
    }

    public close() {
        return this.cckey.close();
    }
}
