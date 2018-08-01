import { Buffer } from "buffer";

import { AssetOutPoint } from "./AssetOutPoint";

export type AssetTransferInputData = {
    prevOut: AssetOutPoint;
    lockScript?: Buffer;
    unlockScript?: Buffer;
};

/**
 * An AssetTransferInput consists of the following:
 *  - An AssetOutPoint, which points to the asset to be spent.
 *  - A lock script and an unlock script, that prove ownership of the asset
 *  - The hashed value(blake256) of a lock script must be identical to that of the pointed asset's lock script hash.
 *  - The results of running the script must return successful in order for the Asset's input to be valid.
 */
export class AssetTransferInput {
    readonly prevOut: AssetOutPoint;
    lockScript: Buffer;
    unlockScript: Buffer;

    /**
     * @param data.prevOut An AssetOutPoint of the input.
     * @param data.lockScript A lock script of the input.
     * @param data.unlockScript A unlock script of the input.
     */
    constructor(data: AssetTransferInputData) {
        const { prevOut, lockScript = Buffer.from([]), unlockScript = Buffer.from([]) } = data;
        this.prevOut = prevOut;
        this.lockScript = Buffer.from(lockScript);
        this.unlockScript = Buffer.from(unlockScript);
    }

    /**
     * Convert to an object for RLP encoding.
     */
    toEncodeObject() {
        const { prevOut, lockScript, unlockScript } = this;
        return [prevOut.toEncodeObject(), lockScript, unlockScript];
    }

    /**
     * Create an AssetTransferInput from an AssetTransferInput JSON object.
     * @param data An AssetTransferInput JSON object.
     * @returns An AssetTransferInput.
     */
    static fromJSON(data: any) {
        const { prevOut, lockScript, unlockScript } = data;
        return new this({
            prevOut: AssetOutPoint.fromJSON(prevOut),
            lockScript,
            unlockScript,
        });
    }

    /**
     * Convert to an AssetTransferInput JSON object.
     * @returns An AssetTransferInput JSON object.
     */
    toJSON() {
        const { prevOut, lockScript, unlockScript } = this;
        return {
            prevOut: prevOut.toJSON(),
            lockScript,
            unlockScript,
        };
    }

    /**
     * Clone a new AssetTransferInput that has empty lock script and empty
     * unlock script. The cloned object is used to sign a transaction.
     * @returns An AssetTransferInput.
     */
    withoutScript() {
        const { prevOut } = this;
        return new AssetTransferInput({
            prevOut,
            lockScript: Buffer.from([]),
            unlockScript: Buffer.from([]),
        });
    }

    /**
     * Set a lock script.
     * @param lockScript A lock script.
     */
    setLockScript(lockScript: Buffer) {
        this.lockScript = lockScript;
    }

    /**
     * Set a unlock script.
     * @param unlockScript A unlock script.
     */
    setUnlockScript(unlockScript: Buffer) {
        this.unlockScript = unlockScript;
    }
}
