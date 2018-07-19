import * as _ from "lodash";
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
    private keyStore: MemoryRawKeyStore;
    private publicKeyMap: { [publicKeyHash: string]: string } = {};

    // FIXME: rename keyStore to rawKeyStore
    constructor(params: { keyStore: MemoryRawKeyStore }) {
        this.keyStore = params.keyStore;
    }

    async createAssetTransferAddress(): Promise<AssetTransferAddress> {
        const publicKey = await this.keyStore.createKey();
        const publicKeyHash = H256.ensure(blake256(publicKey));
        this.publicKeyMap[publicKeyHash.value] = publicKey;
        return AssetTransferAddress.fromTypeAndPayload(1, publicKeyHash);
    }

    async isUnlockable(asset: Asset): Promise<boolean> {
        if (asset.lockScriptHash.value !== blake256(this.generateLockScript())) {
            return false;
        }
        return !!this.publicKeyMap[toHex(asset.parameters[0])];
    }

    async unlock(asset: Asset, tx: AssetTransferTransaction): Promise<{ lockScript: Buffer, unlockScript: Buffer }> {
        const publicKeyList = await this.keyStore.getKeyList();
        const publicKeyHashList = publicKeyList.map(blake256);
        const foundIndex = publicKeyHashList.findIndex((hash) => hash === Buffer.from(asset.parameters[0]).toString("hex"));
        if (foundIndex < 0) {
            throw "Unknown public key hash";
        }
        const publicKey = publicKeyList[foundIndex];
        return {
            lockScript: this.generateLockScript(),
            unlockScript: await this.generateUnlockScript(publicKey, tx.hashWithoutScript()),
        };
    }

    private async generateUnlockScript(publicKey: string, txhash: H256): Promise<Buffer> {
        const { r, s, v } = await this.keyStore.sign({ publicKey, message: txhash.value });
        const signature = new Buffer(65);
        signature.write(_.padStart(r, 64, "0"), 0, 32, "hex");
        signature.write(_.padStart(s, 64, "0"), 32, 32, "hex");
        signature.write(_.padStart(v.toString(16), 2, "0"), 64, 1, "hex");

        const { PUSHB } = Script.Opcode;
        return Buffer.from([PUSHB, 65, ...signature, PUSHB, 64, ...Buffer.from(publicKey, "hex")]);
    }

    private generateLockScript(): Buffer {
        return Script.getStandardScript();
    }
}
