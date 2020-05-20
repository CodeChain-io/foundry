import { Core } from "./core";
import { NetworkId } from "./core/types";
import { Key, KeyStoreType } from "./key";
import {
    blake128,
    blake128WithKey,
    blake160,
    blake160WithKey,
    blake256,
    blake256WithKey,
    generatePrivateKey,
    getPublicFromPrivate,
    ripemd160,
    signEd25519,
    verifyEd25519
} from "./utils";

class SDK {
    public static Core = Core;
    public static Key = Key;
    public static util = {
        blake128,
        blake128WithKey,
        blake160,
        blake160WithKey,
        blake256,
        blake256WithKey,
        ripemd160,
        signEd25519,
        verifyEd25519,
        generatePrivateKey,
        getPublicFromPrivate
    };

    public static SDK = SDK;
    public core: Core;
    public key: Key;
    public util = SDK.util;
    private _networkId: string;

    /**
     * @param params.keyStoreType Specify the type of the keystore. The default value is "local". It creates keystore.db file on the working directory.
     * @param params.networkId The network id of CodeChain. The default value is "tc" (testnet)
     */
    constructor(params: {
        keyStoreType?: KeyStoreType;
        networkId?: NetworkId;
    }) {
        const { keyStoreType = "local", networkId = "tc" } = params;

        this.core = new Core({ networkId });
        this.key = new Key({
            networkId,
            keyStoreType
        });
        this._networkId = networkId;
    }

    public get networkId() {
        return this._networkId;
    }
}

export { SDK };

module.exports = SDK;
