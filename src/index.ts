import {
    blake256, blake256WithKey, ripemd160, signEcdsa,
    getPublicFromPrivate, getAccountIdFromPrivate, verifyEcdsa, recoverEcdsa,
    generatePrivateKey
} from "./utils";
import { Rpc } from "./rpc";
import { Core } from "./core";

class SDK {
    public rpc: Rpc;
    public core: Core;
    public util = SDK.util;
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

    /**
     * @param params.server HTTP RPC server address
     * @param params.networkId The network id of CodeChain. The default value is 0x11 (solo consensus)
     */
    constructor(params: { server: string, networkId?: number }) {
        const { server, networkId = 0x11 } = params;

        this.rpc = new Rpc({ server });
        this.core = new Core({ networkId });
    }
}

export { SDK };

module.exports = SDK;
