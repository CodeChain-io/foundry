"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const core_1 = require("./core");
const key_1 = require("./key");
const rpc_1 = require("./rpc");
const utils_1 = require("./utils");
class SDK {
    /**
     * @param params.server HTTP RPC server address
     * @param params.keyStoreType Specify the type of the keystore. The default value is "local". It creates keystore.db file on the working directory.
     * @param params.networkId The network id of CodeChain. The default value is "tc" (testnet)
     */
    constructor(params) {
        this.util = SDK.util;
        const { server, keyStoreType = "local", networkId = "tc", options } = params;
        const { networkId: networkIdOpt, parcelSigner, parcelFee = 10 } = options || { networkId: undefined, parcelSigner: undefined };
        this.rpc = new rpc_1.Rpc({ server, options: { parcelSigner, parcelFee } });
        this.core = new core_1.Core({ networkId: networkIdOpt || networkId });
        this.key = new key_1.Key({
            networkId: networkIdOpt || networkId,
            keyStoreType
        });
    }
}
SDK.Rpc = rpc_1.Rpc;
SDK.Core = core_1.Core;
SDK.Key = key_1.Key;
SDK.util = {
    blake128: utils_1.blake128,
    blake128WithKey: utils_1.blake128WithKey,
    blake160: utils_1.blake160,
    blake160WithKey: utils_1.blake160WithKey,
    blake256: utils_1.blake256,
    blake256WithKey: utils_1.blake256WithKey,
    ripemd160: utils_1.ripemd160,
    signEcdsa: utils_1.signEcdsa,
    verifyEcdsa: utils_1.verifyEcdsa,
    recoverEcdsa: utils_1.recoverEcdsa,
    generatePrivateKey: utils_1.generatePrivateKey,
    getAccountIdFromPrivate: utils_1.getAccountIdFromPrivate,
    getPublicFromPrivate: utils_1.getPublicFromPrivate
};
SDK.SDK = SDK;
exports.SDK = SDK;
module.exports = SDK;
