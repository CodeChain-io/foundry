/// <reference types="node" />
import { AssetOutPoint } from "./AssetOutPoint";
export declare type TimelockType = "block" | "blockAge" | "time" | "timeAge";
export interface Timelock {
    type: TimelockType;
    value: number;
}
/**
 * An AssetTransferInput consists of the following:
 *  - An AssetOutPoint, which points to the asset to be spent.
 *  - A lock script and an unlock script, that prove ownership of the asset
 *  - The hashed value(blake160) of a lock script must be identical to that of the pointed asset's lock script hash.
 *  - The results of running the script must return successful in order for the Asset's input to be valid.
 */
export declare class AssetTransferInput {
    /**
     * Create an AssetTransferInput from an AssetTransferInput JSON object.
     * @param data An AssetTransferInput JSON object.
     * @returns An AssetTransferInput.
     */
    static fromJSON(data: any): AssetTransferInput;
    readonly prevOut: AssetOutPoint;
    readonly timelock: Timelock | null;
    lockScript: Buffer;
    unlockScript: Buffer;
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
    });
    /**
     * Convert to an object for RLP encoding.
     */
    toEncodeObject(): ((string | number)[] | Buffer | number[][])[];
    /**
     * Convert to an AssetTransferInput JSON object.
     * @returns An AssetTransferInput JSON object.
     */
    toJSON(): {
        prevOut: {
            transactionHash: string;
            index: number;
            assetType: string;
            amount: string | number;
        };
        timelock: Timelock | null;
        lockScript: number[];
        unlockScript: number[];
    };
    /**
     * Clone a new AssetTransferInput that has empty lock script and empty
     * unlock script. The cloned object is used to sign a transaction.
     * @returns An AssetTransferInput.
     */
    withoutScript(): AssetTransferInput;
    /**
     * Set a lock script.
     * @param lockScript A lock script.
     */
    setLockScript(lockScript: Buffer): void;
    /**
     * Set a unlock script.
     * @param unlockScript A unlock script.
     */
    setUnlockScript(unlockScript: Buffer): void;
}
