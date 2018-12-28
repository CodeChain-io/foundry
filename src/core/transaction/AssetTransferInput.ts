import { Buffer } from "buffer";

import { AssetOutPoint, AssetOutPointJSON } from "./AssetOutPoint";

export type TimelockType = "block" | "blockAge" | "time" | "timeAge";
export interface Timelock {
    type: TimelockType;
    // FIXME: U64
    value: number;
}

export interface AssetTransferInputJSON {
    prevOut: AssetOutPointJSON;
    timelock: Timelock | null;
    lockScript: number[];
    unlockScript: number[];
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
    public static fromJSON(data: AssetTransferInputJSON) {
        const { prevOut, timelock, lockScript, unlockScript } = data;
        return new AssetTransferInput({
            prevOut: AssetOutPoint.fromJSON(prevOut),
            timelock,
            lockScript: Buffer.from(lockScript),
            unlockScript: Buffer.from(unlockScript)
        });
    }
    public readonly prevOut: AssetOutPoint;
    public readonly timelock: Timelock | null;
    public lockScript: Buffer;
    public unlockScript: Buffer;

    /**
     * @param data.prevOut An AssetOutPoint of the input.
     * @param data.lockScript A lock script of the input.
     * @param data.unlockScript A unlock script of the input.
     */
    constructor(data: {
        prevOut: AssetOutPoint;
        timelock: Timelock | null;
        lockScript?: Buffer;
        unlockScript?: Buffer;
    }) {
        const {
            prevOut,
            timelock,
            lockScript = Buffer.from([]),
            unlockScript = Buffer.from([])
        } = data;
        this.prevOut = prevOut;
        this.timelock = timelock;
        this.lockScript = Buffer.from(lockScript);
        this.unlockScript = Buffer.from(unlockScript);
    }

    /**
     * Convert to an object for RLP encoding.
     */
    public toEncodeObject() {
        const { prevOut, timelock, lockScript, unlockScript } = this;
        return [
            prevOut.toEncodeObject(),
            convertTimelockToEncodeObject(timelock),
            lockScript,
            unlockScript
        ];
    }

    /**
     * Convert to an AssetTransferInput JSON object.
     * @returns An AssetTransferInput JSON object.
     */
    public toJSON(): AssetTransferInputJSON {
        const { prevOut, timelock, lockScript, unlockScript } = this;
        return {
            prevOut: prevOut.toJSON(),
            timelock,
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
        const { prevOut, timelock } = this;
        return new AssetTransferInput({
            prevOut,
            timelock,
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

function convertTimelockToEncodeObject(timelock: Timelock | null) {
    if (timelock === null) {
        return [];
    }
    const { type, value } = timelock;
    let typeEncoded;
    switch (type) {
        case "block":
            typeEncoded = 1;
            break;
        case "blockAge":
            typeEncoded = 2;
            break;
        case "time":
            typeEncoded = 3;
            break;
        case "timeAge":
            typeEncoded = 4;
            break;
        default:
            throw Error(`Unexpected timelock type: ${type}`);
    }
    return [[typeEncoded, value]];
}
