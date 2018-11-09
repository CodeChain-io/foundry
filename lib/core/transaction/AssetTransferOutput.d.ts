/// <reference types="node" />
import { AssetTransferAddress, H160 } from "codechain-primitives";
import { H256 } from "../H256";
import { U256 } from "../U256";
export interface AssetTransferOutputData {
    lockScriptHash: H160;
    parameters: Buffer[];
    assetType: H256;
    amount: U256;
}
export interface AssetTransferOutputAddressData {
    recipient: AssetTransferAddress;
    assetType: H256;
    amount: U256;
}
/**
 * An AssetTransferOutput consists of:
 *  - A lock script hash and parameters, which mark ownership of the asset.
 *  - An asset type and amount, which indicate the asset's type and quantity.
 */
export declare class AssetTransferOutput {
    /**
     * Create an AssetTransferOutput from an AssetTransferOutput JSON object.
     * @param data An AssetTransferOutput JSON object.
     * @returns An AssetTransferOutput.
     */
    static fromJSON(data: AssetTransferOutputData): AssetTransferOutput;
    readonly lockScriptHash: H160;
    readonly parameters: Buffer[];
    readonly assetType: H256;
    readonly amount: U256;
    /**
     * @param data.lockScriptHash A lock script hash of the output.
     * @param data.parameters Parameters of the output.
     * @param data.assetType An asset type of the output.
     * @param data.amount An asset amount of the output.
     */
    constructor(data: AssetTransferOutputData | AssetTransferOutputAddressData);
    /**
     * Convert to an object for RLP encoding.
     */
    toEncodeObject(): (string | number | Buffer[])[];
    /**
     * Convert to an AssetTransferOutput JSON object.
     * @returns An AssetTransferOutput JSON object.
     */
    toJSON(): {
        lockScriptHash: string;
        parameters: number[][];
        assetType: string;
        amount: string | number;
    };
    /**
     * Get the shard ID.
     * @returns A shard ID.
     */
    shardId(): number;
}
