import { Rpc } from "../rpc";
import { AssetTransferTransaction, Parcel, SignedParcel, H160 } from "../core/classes";

import { MemoryRawKeyStore } from "./MemoryRawKeyStore";
import { AssetTransferAddress } from "./AssetTransferAddress";
import { PlatformAddress } from "./PlatformAddress";
import { PkhAssetAgent } from "./PkhAssetAgent";

/**
 * @hidden
 */
export type AssetAgent = PkhAssetAgent;

export class Key {
    private rpc: Rpc;
    private pkhAssetAgent: PkhAssetAgent;

    constructor(rpc: Rpc) {
        this.rpc = rpc;
        this.pkhAssetAgent = new PkhAssetAgent({ keyStore: new MemoryRawKeyStore() });
    }

    /**
     * Creates MemoryRawKeyStore which is a simple key store for testing purpose. Do
     * not use this in production.
     * @returns new instance of MemoryRawKeyStore
     */
    createMemoryRawKeyStore(): MemoryRawKeyStore {
        return new MemoryRawKeyStore();
    }

    /**
     * Creates AssetTransferAddress for the standard P2PKH asset.
     * To use this address, see AssetScheme.createMintTransaction() or Asset.transfer().
     * @returns AssetTransferAddress
     */
    createPubKeyHashAddress(): Promise<AssetTransferAddress> {
        return this.pkhAssetAgent.createAddress();
    }

    /**
     * Signs a Parcel with the given account.
     * @param parcel A Parcel
     * @param params.account An account.
     * @param params.passphrase The passphrase for the given account
     * @returns A SignedParcel
     * @throws When nonce or fee in the Parcel is null
     * @throws When account or its passphrase is invalid
     */
    async signParcel(parcel: Parcel, params: { account: H160 | string, passphrase?: string }): Promise<SignedParcel> {
        const { account, passphrase } = params;
        const sig = await this.rpc.account.sign(parcel.hash(), account, passphrase);
        return new SignedParcel(parcel, sig);
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

        if (await this.pkhAssetAgent.isUnlockable(asset)) {
            const { unlockScript, lockScript } = await this.pkhAssetAgent.unlock(asset, transaction);
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
        MemoryRawKeyStore,
    };
}
