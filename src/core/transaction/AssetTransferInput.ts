import { Buffer } from "buffer";

import { AssetOutPoint } from "./AssetOutPoint";

export interface AssetTransferInputData {
    prevOut: AssetOutPoint;
    lockScript?: Buffer;
    unlockScript?: Buffer;
}

/**
 * An AssetTransferInput consists of the following:
 *  - An AssetOutPoint, which points to the asset to be spent.
 *  - A lock script and an unlock script, that prove ownership of the asset
 *  - The hashed value(blake160) of a lock script must be identical to that of the pointed asset's lock script hash.
 *  - The results of running the script must return successful in order for the Asset's input to be valid.
 */
export class AssetTransferInput {
    /**
     * Create an AssetTransferInput from an AssetTransferInput JSON object.
     * @param data An AssetTransferInput JSON object.
     * @returns An AssetTransferInput.
     */
    public static fromJSON(data: any) {
        const { prevOut, lockScript, unlockScript } = data;
        return new this({
            prevOut: AssetOutPoint.fromJSON(prevOut),
            lockScript,
            unlockScript
        });
    }
    public readonly prevOut: AssetOutPoint;
    public lockScript: Buffer;
    public unlockScript: Buffer;

    /**
     * @param data.prevOut An AssetOutPoint of the input.
     * @param data.lockScript A lock script of the input.
     * @param data.unlockScript A unlock script of the input.
     */
    constructor(data: AssetTransferInputData) {
        const {
            prevOut,
            lockScript = Buffer.from([]),
            unlockScript = Buffer.from([])
        } = data;
        this.prevOut = prevOut;
        this.lockScript = Buffer.from(lockScript);
        this.unlockScript = Buffer.from(unlockScript);
    }

    /**
     * Convert to an object for RLP encoding.
     */
    public toEncodeObject() {
        const { prevOut, lockScript, unlockScript } = this;
        return [prevOut.toEncodeObject(), lockScript, unlockScript];
    }

    /**
     * Convert to an AssetTransferInput JSON object.
     * @returns An AssetTransferInput JSON object.
     */
    public toJSON() {
        const { prevOut, lockScript, unlockScript } = this;
        return {
            prevOut: prevOut.toJSON(),
            lockScript: [...lockScript],
            unlockScript: [...unlockScript]
        };
    }

    /**
     * Clone a new AssetTransferInput that has empty lock script and empty
     * unlock script. The cloned object is used to sign a transaction.
     * @returns An AssetTransferInput.
     */
    public withoutScript() {
        const { prevOut } = this;
        return new AssetTransferInput({
            prevOut,
            lockScript: Buffer.from([]),
            unlockScript: Buffer.from([])
        });
    }

    /**
     * Set a lock script.
     * @param lockScript A lock script.
     */
    public setLockScript(lockScript: Buffer) {
        this.lockScript = lockScript;
    }

    /**
     * Set a unlock script.
     * @param unlockScript A unlock script.
     */
    public setUnlockScript(unlockScript: Buffer) {
        this.unlockScript = unlockScript;
    }
}
