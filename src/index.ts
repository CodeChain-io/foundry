import { Core } from "./core";
import { NetworkId } from "./core/types";
import { Key, KeyStoreType } from "./key";
import { Rpc } from "./rpc";
import {
    blake128,
    blake128WithKey,
    blake160,
    blake160WithKey,
    blake256,
    blake256WithKey,
    generatePrivateKey,
    getAccountIdFromPrivate,
    getPublicFromPrivate,
    recoverEcdsa,
    ripemd160,
    signEcdsa,
    verifyEcdsa
} from "./utils";

class SDK {
    public static Rpc = Rpc;
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
        signEcdsa,
        verifyEcdsa,
        recoverEcdsa,
        generatePrivateKey,
        getAccountIdFromPrivate,
        getPublicFromPrivate
    };

    public static SDK = SDK;
    public rpc: Rpc;
    public core: Core;
    public key: Key;
    public util = SDK.util;
    private _networkId: string;

    /**
     * @param params.server HTTP RPC server address
     * @param params.keyStoreType Specify the type of the keystore. The default value is "local". It creates keystore.db file on the working directory.
     * @param params.networkId The network id of CodeChain. The default value is "tc" (testnet)
     */
    constructor(params: {
        server: string;
        keyStoreType?: KeyStoreType;
        networkId?: NetworkId;
        // Deprecated. It will be removed at 0.2.0
        options?: {
            transactionSigner?: string;
            transactionFee?: number;
        };
    }) {
        const {
            server,
            keyStoreType = "local",
            networkId = "tc",
            options
        } = params;
        const { transactionSigner, transactionFee = 10 } = options || {
            transactionSigner: undefined
        };

        this.rpc = new Rpc({
            server,
            options: { transactionSigner, transactionFee }
        });
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
