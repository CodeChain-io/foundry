import { Core } from "./core";
import { NetworkId } from "./core/types";
import { Key, KeyStoreType } from "./key";
import { Rpc } from "./rpc";
import {
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

    /**
     * @param params.server HTTP RPC server address
     * @param params.keyStoreType Specify the type of the keystore. The default value is "local". It creates keystore.db file on the working directory.
     * @param params.options.networkId The network id of CodeChain. The default value is "tc" (testnet)
     * @param params.options.parcelSigner The default account to sign the parcel
     * @param params.options.parcelFee The default amount for the parcel fee
     */
    constructor(params: {
        server: string;
        keyStoreType?: KeyStoreType;
        options?: {
            networkId?: NetworkId;
            parcelSigner?: string;
            parcelFee?: number;
        };
    }) {
        const { server, keyStoreType = "local", options = {} } = params;
        const { networkId = "tc", parcelSigner, parcelFee = 10 } = options;

        this.rpc = new Rpc({ server, options: { parcelSigner, parcelFee } });
        this.core = new Core({ networkId });
        this.key = new Key({ networkId, keyStoreType });
    }
}

export { SDK };

module.exports = SDK;
