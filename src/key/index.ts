import { Rpc } from "../rpc";
import { AssetTransferTransaction } from "../core/transaction/AssetTransferTransaction";

import { MemoryKeyStore } from "./MemoryKeyStore";
import { PubkeyAssetAgent } from "./PubkeyAssetAgent";
import { AssetTransferAddress } from "./AssetTransferAddress";
import { PlatformAddress } from "./PlatformAddress";
import { PkhAssetAgent } from "./PkhAssetAgent";

/**
 * hidden
 */
export type KeyStore = MemoryKeyStore;

/**
 * @hidden
 */
export type AssetAgent = PubkeyAssetAgent | PkhAssetAgent;

export class Key {
    private rpc: Rpc;
    private pkAssetAgent: PubkeyAssetAgent;
    private pkhAssetAgent: PkhAssetAgent;

    constructor(rpc: Rpc) {
        this.rpc = rpc;
        this.pkAssetAgent = new PubkeyAssetAgent({ keyStore: new MemoryKeyStore() });
        this.pkhAssetAgent = new PkhAssetAgent({ keyStore: new MemoryKeyStore() });
    }

    /**
     * Creates AssetTransferAddress for non-standard P2PK asset.
     * To use this address AssetScheme.mint() or Asset.transfer().
     * @returns AssetTransferAddress
     */
    createPubKeyAddress(): Promise<AssetTransferAddress> {
        return this.pkAssetAgent.createAddress();
    }

    /**
     * Creates AssetTransferAddress for the standard P2PKH asset.
     * To use this address, see AssetScheme.mint() or Asset.transfer().
     * @returns AssetTransferAddress
     */
    createPubKeyHashAddress(): Promise<AssetTransferAddress> {
        return this.pkhAssetAgent.createAddress();
    }

    /**
     * Sets lock and unlock scripts to the given transaction's input. The unlock
     * script contains the signature for the whole tranasaction except for scripts
     * in it.
     * @param transaction AssetTransferTransaction to sign
     * @param inputIndex An index of input to unlock
     * @returns True if successful, false if unable to recognize lock script hash or
     * unable to create the signature
     * @throws When the input is already spent or never has been exist
     * @throws When the given index is out of range
     */
    async unlock(transaction: AssetTransferTransaction, inputIndex: number): Promise<boolean> {
        if (inputIndex >= transaction.inputs.length) {
            throw "Invalid input index.";
        }
        const asset = await this.rpc.chain.getAsset(transaction.inputs[inputIndex].prevOut.transactionHash, inputIndex);
        if (asset === null) {
            throw "Asset is not exist or spent.";
        }

        if (await this.pkhAssetAgent.inUnlockable(asset)) {
            const { unlockScript, lockScript } = await this.pkhAssetAgent.unlock(asset, transaction);
            transaction.setLockScript(inputIndex, lockScript);
            transaction.setUnlockScript(inputIndex, unlockScript);
        } else if (await this.pkAssetAgent.isUnlockable(asset)) {
            const { unlockScript, lockScript } = await this.pkAssetAgent.unlock(asset, transaction);
            transaction.setLockScript(inputIndex, lockScript);
            transaction.setUnlockScript(inputIndex, unlockScript);
        } else {
            return false;
        }
        return true;
    }

    public classes = Key.classes;
    static classes = {
        AssetTransferAddress,
        PlatformAddress,
        PubkeyAssetAgent,
        MemoryKeyStore,
    };
}
