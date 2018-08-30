import { Core } from "./core";
import { Key } from "./key";
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

type NetworkId = string;

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
     * @param params.options.networkId The network id of CodeChain. The default value is "tc" (testnet)
     * @param params.options.parcelSigner The default account to sign the parcel
     * @param params.options.parcelFee The default amount for the parcel fee
     */
    constructor(params: {
        server: string;
        options?: {
            networkId?: NetworkId;
            parcelSigner?: string;
            parcelFee?: number;
        };
    }) {
        const { server, options = {} } = params;
        const { networkId = "tc", parcelSigner, parcelFee = 10 } = options;

        this.rpc = new Rpc({ server, options: { parcelSigner, parcelFee } });
        this.core = new Core({ networkId });
        this.key = new Key(this.rpc, { networkId });
    }
}

export { SDK };

module.exports = SDK;
