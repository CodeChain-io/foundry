/// <reference types="node" />
import { Core } from "./core";
import { NetworkId } from "./core/types";
import { Key, KeyStoreType } from "./key";
import { Rpc } from "./rpc";
declare class SDK {
    static Rpc: typeof Rpc;
    static Core: typeof Core;
    static Key: typeof Key;
    static util: {
        blake128: (data: string | Buffer) => string;
        blake128WithKey: (data: string | Buffer, key: Uint8Array) => string;
        blake160: (data: string | Buffer) => string;
        blake160WithKey: (data: string | Buffer, key: Uint8Array) => string;
        blake256: (data: string | Buffer) => string;
        blake256WithKey: (data: string | Buffer, key: Uint8Array) => string;
        ripemd160: (data: string | Buffer) => string;
        signEcdsa: (message: string, priv: string) => import("./utils").EcdsaSignature;
        verifyEcdsa: (message: string, signature: import("./utils").EcdsaSignature, pub: string) => boolean;
        recoverEcdsa: (message: string, signature: import("./utils").EcdsaSignature) => string;
        generatePrivateKey: () => string;
        getAccountIdFromPrivate: (priv: string) => string;
        getPublicFromPrivate: (priv: string) => string;
    };
    static SDK: typeof SDK;
    rpc: Rpc;
    core: Core;
    key: Key;
    util: {
        blake128: (data: string | Buffer) => string;
        blake128WithKey: (data: string | Buffer, key: Uint8Array) => string;
        blake160: (data: string | Buffer) => string;
        blake160WithKey: (data: string | Buffer, key: Uint8Array) => string;
        blake256: (data: string | Buffer) => string;
        blake256WithKey: (data: string | Buffer, key: Uint8Array) => string;
        ripemd160: (data: string | Buffer) => string;
        signEcdsa: (message: string, priv: string) => import("./utils").EcdsaSignature;
        verifyEcdsa: (message: string, signature: import("./utils").EcdsaSignature, pub: string) => boolean;
        recoverEcdsa: (message: string, signature: import("./utils").EcdsaSignature) => string;
        generatePrivateKey: () => string;
        getAccountIdFromPrivate: (priv: string) => string;
        getPublicFromPrivate: (priv: string) => string;
    };
    /**
     * @param params.server HTTP RPC server address
     * @param params.keyStoreType Specify the type of the keystore. The default value is "local". It creates keystore.db file on the working directory.
     * @param params.networkId The network id of CodeChain. The default value is "tc" (testnet)
     */
    constructor(params: {
        server: string;
        keyStoreType?: KeyStoreType;
        networkId?: NetworkId;
        options?: {
            networkId?: NetworkId;
            parcelSigner?: string;
            parcelFee?: number;
        };
    });
}
export { SDK };
