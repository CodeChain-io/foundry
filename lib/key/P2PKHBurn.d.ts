/// <reference types="node" />
import { AssetTransferAddress, H160 } from "codechain-primitives";
import { H256 } from "../core/H256";
import { NetworkId } from "../core/types";
import { SignatureTag } from "../utils";
import { KeyStore } from "./KeyStore";
export declare class P2PKHBurn {
    static getLockScript(): Buffer;
    static getLockScriptHash(): H160;
    private keyStore;
    private networkId;
    constructor(params: {
        keyStore: KeyStore;
        networkId: NetworkId;
    });
    createAddress(options?: {
        passphrase?: string;
    }): Promise<AssetTransferAddress>;
    createUnlockScript(publicKeyHash: string, txhash: H256, options?: {
        passphrase?: string;
        signatureTag?: SignatureTag;
    }): Promise<Buffer>;
}
