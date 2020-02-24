import { CCKey } from "codechain-keystore";
import * as _ from "lodash";
import { KeyStore } from "./KeyStore";

// type of key is AccountId | PublicKeyHash
export interface HDKeyMapping {
    platform: {
        [key: string]: {
            seedHash: string;
            path: string;
        };
    };
    asset: {
        [key: string]: {
            seedHash: string;
            path: string;
        };
    };
}

export class LocalKeyStore implements KeyStore {
    public static async create(
        options: { dbPath?: string; mapping?: HDKeyMapping } = {}
    ): Promise<KeyStore> {
        const cckey = await CCKey.create(options);
        return new LocalKeyStore(cckey, options.mapping);
    }

    public static async createForTest(): Promise<KeyStore> {
        const cckey = await CCKey.create({ dbType: "in-memory" });
        return new LocalKeyStore(cckey);
    }
    public cckey: CCKey;

    public platform = {
        getKeyList: async (): Promise<string[]> => {
            const hdKeys = _.keys(this.hdKeyMapping.platform);
            const keys = await this.cckey.platform.getKeys();
            return [...hdKeys, ...keys];
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
            if (this.hdKeyMapping.platform[params.key]) {
                const { seedHash, path } = this.hdKeyMapping.platform[
                    params.key
                ];
                return this.cckey.hdwseed.getPublicKeyFromSeed({
                    seedHash,
                    path,
                    passphrase
                });
            } else {
                return this.cckey.platform.getPublicKey({ key, passphrase });
            }
        },

        sign: (params: {
            key: string;
            message: string;
            passphrase?: string;
        }): Promise<string> => {
            const { passphrase = "" } = params;
            if (this.hdKeyMapping.platform[params.key]) {
                const { seedHash, path } = this.hdKeyMapping.platform[
                    params.key
                ];
                return this.cckey.hdwseed.signFromSeed({
                    seedHash,
                    path,
                    passphrase,
                    message: params.message
                });
            } else {
                return this.cckey.platform.sign({ ...params, passphrase });
            }
        }
    };

    public asset = {
        getKeyList: async (): Promise<string[]> => {
            const hdKeys = _.keys(this.hdKeyMapping.asset);
            const keys = await this.cckey.asset.getKeys();
            return [...hdKeys, ...keys];
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
            if (this.hdKeyMapping.asset[params.key]) {
                const { seedHash, path } = this.hdKeyMapping.asset[params.key];
                return this.cckey.hdwseed.getPublicKeyFromSeed({
                    seedHash,
                    path,
                    passphrase
                });
            } else {
                return this.cckey.asset.getPublicKey({ key, passphrase });
            }
        },

        sign: (params: {
            key: string;
            message: string;
            passphrase?: string;
        }): Promise<string> => {
            const { passphrase = "" } = params;
            if (this.hdKeyMapping.asset[params.key]) {
                const { seedHash, path } = this.hdKeyMapping.asset[params.key];
                return this.cckey.hdwseed.signFromSeed({
                    seedHash,
                    path,
                    passphrase,
                    message: params.message
                });
            } else {
                return this.cckey.asset.sign({ ...params, passphrase });
            }
        }
    };

    private hdKeyMapping: HDKeyMapping;

    public constructor(cckey: CCKey, hdKeyMapping?: HDKeyMapping) {
        this.cckey = cckey;
        this.hdKeyMapping = hdKeyMapping || {
            platform: {},
            asset: {}
        };
    }

    public close() {
        return this.cckey.close();
    }
}
