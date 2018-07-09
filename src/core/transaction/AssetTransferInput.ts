import { AssetOutPoint } from "./AssetOutPoint";

export type AssetTransferInputData = {
    prevOut: AssetOutPoint;
    lockScript: Buffer;
    unlockScript: Buffer;
};
/**
 * AssetTransferInput consists of the following:
 *
 * - AssetOutPoint, which points to the asset to be spent.
 * - lockScript and unlockScript, that prove ownership of the asset
 * - The hashed value(blake256) of lockScript must be identical to that of the pointed asset's lockScriptHash.
 * - The results of running the script must return successful in order for the Asset's Input to be valid.
 */
export class AssetTransferInput {
    private readonly prevOut: AssetOutPoint;
    private lockScript: Buffer;
    private unlockScript: Buffer;

    constructor(data: AssetTransferInputData) {
        const { prevOut, lockScript, unlockScript } = data;
        this.prevOut = prevOut;
        this.lockScript = lockScript;
        this.unlockScript = unlockScript;
    }

    toEncodeObject() {
        const { prevOut, lockScript, unlockScript } = this;
        return [prevOut.toEncodeObject(), lockScript, unlockScript];
    }

    static fromJSON(data: any) {
        const { prevOut, lockScript, unlockScript } = data;
        return new this({
            prevOut: AssetOutPoint.fromJSON(prevOut),
            lockScript,
            unlockScript,
        });
    }

    toJSON() {
        const { prevOut, lockScript, unlockScript } = this;
        return {
            prevOut: prevOut.toJSON(),
            lockScript,
            unlockScript,
        };
    }

    withoutScript() {
        const { prevOut } = this;
        return new AssetTransferInput({
            prevOut,
            lockScript: Buffer.from([]),
            unlockScript: Buffer.from([]),
        });
    }

    setLockScript(lockScript: Buffer) {
        this.lockScript = lockScript;
    }

    setUnlockScript(unlockScript: Buffer) {
        this.unlockScript = unlockScript;
    }
}
