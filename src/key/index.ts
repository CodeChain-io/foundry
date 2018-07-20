import { Rpc } from "../rpc";
import { AssetTransferTransaction, Parcel, SignedParcel, H160 } from "../core/classes";

import { MemoryKeyStore } from "./MemoryKeyStore";
import { AssetTransferAddress } from "./AssetTransferAddress";
import { PlatformAddress } from "./PlatformAddress";
import { P2PKH } from "./P2PKH";

type KeyStore = MemoryKeyStore;

export class Key {
    private rpc: Rpc;
    private memoryKeyStore: P2PKH;

    constructor(rpc: Rpc) {
        this.rpc = rpc;
        this.memoryKeyStore = new P2PKH({ keyStore: new MemoryKeyStore() });
    }

    /**
     * Creates key store which is non-persistent. Do not use in production.
     */
    createMemoryKeyStore(): MemoryKeyStore {
        return new MemoryKeyStore();
    }

    /**
     * Creates P2PKH script generator.
     * @returns new instance of P2PKH
     */
    createP2PKH(params: { keyStore: KeyStore }): P2PKH {
        const { keyStore } = params;
        return new P2PKH({ keyStore });
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

        if (await this.memoryKeyStore.isUnlockable(asset)) {
            const { unlockScript, lockScript } = await this.memoryKeyStore.unlock(asset, transaction);
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
        MemoryKeyStore,
    };
}
