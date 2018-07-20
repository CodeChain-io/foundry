import { Buffer } from "buffer";

import { Asset } from "../core/Asset";
import { H256 } from "../core/H256";
import { AssetTransferTransaction } from "../core/transaction/AssetTransferTransaction";
import { Script } from "../core/Script";
import { blake256, toHex } from "../utils";

import { AssetTransferAddress } from "./AssetTransferAddress";
import { MemoryRawKeyStore } from "./MemoryRawKeyStore";

/**
 * AssetAgent which supports P2PKH(Pay to Public Key Hash).
 */
export class MemoryKeyStore {
    private rawKeyStore: MemoryRawKeyStore;
    private publicKeyMap: { [publicKeyHash: string]: string } = {};

    // FIXME: rename keyStore to rawKeyStore
    constructor(params: { keyStore: MemoryRawKeyStore }) {
        this.rawKeyStore = params.keyStore;
    }

    async createAssetTransferAddress(): Promise<AssetTransferAddress> {
        const publicKey = await this.rawKeyStore.createKey();
        const publicKeyHash = H256.ensure(blake256(publicKey));
        this.publicKeyMap[publicKeyHash.value] = publicKey;
        return AssetTransferAddress.fromTypeAndPayload(1, publicKeyHash);
    }

    async isUnlockable(asset: Asset): Promise<boolean> {
        if (this.isStandardLockScriptHash(asset.lockScriptHash)) {
            return false;
        }
        if (asset.parameters.length !== 1 || asset.parameters[0].byteLength !== 32) {
            return false;
        }
        return !!this.publicKeyMap[toHex(asset.parameters[0])];
    }

    async unlock(asset: Asset, tx: AssetTransferTransaction): Promise<{ lockScript: Buffer, unlockScript: Buffer }> {
        const publicKeyHash = Buffer.from(asset.parameters[0]).toString("hex");
        const publicKey = this.publicKeyMap[publicKeyHash];
        if (!publicKey) {
            throw "Unknown public key hash";
        }
        return {
            lockScript: Script.getStandardScript(),
            unlockScript: await this.generateUnlockScript(publicKey, tx.hashWithoutScript()),
        };
    }

    private async generateUnlockScript(publicKey: string, txhash: H256): Promise<Buffer> {
        const signature = await this.rawKeyStore.sign({ publicKey, message: txhash.value });
        const { PUSHB } = Script.Opcode;
        return Buffer.from([
            PUSHB,
            65,
            ...Buffer.from(signature, "hex"),
            PUSHB,
            64,
            ...Buffer.from(publicKey, "hex")
        ]);
    }

    private isStandardLockScriptHash(hash: H256): boolean {
        return hash.value === Script.getStandardScriptHash().value;
    }
}
