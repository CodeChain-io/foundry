/// <reference types="node" />
import { SignatureTag } from "../../utils";
import { Asset } from "../Asset";
import { H256 } from "../H256";
import { AssetTransferOutputValue, NetworkId } from "../types";
import { AssetTransferInput } from "./AssetTransferInput";
import { AssetTransferOutput } from "./AssetTransferOutput";
export interface AssetTransferTransactionData {
    burns: AssetTransferInput[];
    inputs: AssetTransferInput[];
    outputs: AssetTransferOutput[];
    networkId: NetworkId;
}
/**
 * Spends the existing asset and creates a new asset. Ownership can be transferred during this process.
 *
 * An AssetTransferTransaction consists of:
 *  - A list of AssetTransferInput to burn.
 *  - A list of AssetTransferInput to spend.
 *  - A list of AssetTransferOutput to create.
 *  - A network ID. This must be identical to the network ID of which the
 *  transaction is being sent to.
 *
 * All inputs must be valid for the transaction to be valid. When each asset
 * types' amount have been summed, the sum of inputs and the sum of outputs
 * must be identical.
 */
export declare class AssetTransferTransaction {
    /** Create an AssetTransferTransaction from an AssetTransferTransaction JSON object.
     * @param obj An AssetTransferTransaction JSON object.
     * @returns An AssetTransferTransaction.
     */
    static fromJSON(obj: any): AssetTransferTransaction;
    readonly burns: AssetTransferInput[];
    readonly inputs: AssetTransferInput[];
    readonly outputs: AssetTransferOutput[];
    readonly networkId: NetworkId;
    readonly type: string;
    /**
     * @param params.burns An array of AssetTransferInput to burn.
     * @param params.inputs An array of AssetTransferInput to spend.
     * @param params.outputs An array of AssetTransferOutput to create.
     * @param params.networkId A network ID of the transaction.
     */
    constructor(params: AssetTransferTransactionData);
    /**
     * Convert to an object for RLP encoding.
     */
    toEncodeObject(): (string | number | ((string | number)[] | Buffer | number[][])[][] | (string | number | Buffer[])[][])[];
    /**
     * Convert to RLP bytes.
     */
    rlpBytes(): Buffer;
    /**
     * Get the hash of an AssetTransferTransaction.
     * @returns A transaction hash.
     */
    hash(): H256;
    /**
     * Add an AssetTransferInput to burn.
     * @param burns An array of either an AssetTransferInput or an Asset.
     * @returns The AssetTransferTransaction, which is modified by adding them.
     */
    addBurns(burns: AssetTransferInput | Asset | Array<AssetTransferInput | Asset>, ...rest: Array<AssetTransferInput | Asset>): AssetTransferTransaction;
    /**
     * Add an AssetTransferInput to spend.
     * @param inputs An array of either an AssetTransferInput or an Asset.
     * @returns The AssetTransferTransaction, which is modified by adding them.
     */
    addInputs(inputs: AssetTransferInput | Asset | Array<AssetTransferInput | Asset>, ...rest: Array<AssetTransferInput | Asset>): AssetTransferTransaction;
    /**
     * Add an AssetTransferOutput to create.
     * @param outputs An array of either an AssetTransferOutput or an object
     * that has amount, assetType and recipient values.
     * @param output.amount Asset amount of the output.
     * @param output.assetType An asset type of the output.
     * @param output.recipient A recipient of the output.
     */
    addOutputs(outputs: AssetTransferOutputValue | Array<AssetTransferOutputValue>, ...rest: Array<AssetTransferOutputValue>): AssetTransferTransaction;
    /**
     * Get the output of the given index, of this transaction.
     * @param index An index indicating an output.
     * @returns An Asset.
     */
    getTransferredAsset(index: number): Asset;
    /**
     * Get the outputs of this transaction.
     * @returns An array of an Asset.
     */
    getTransferredAssets(): Asset[];
    /**
     * Get a hash of the transaction that doesn't contain the scripts. The hash
     * is used as a message to create a signature for a transaction.
     * @returns A hash.
     */
    hashWithoutScript(params?: {
        tag: SignatureTag;
        type: "input" | "burn";
        index: number;
    }): H256;
    /**
     * Get the asset address of an output.
     * @param index An index indicating the output.
     * @returns An asset address which is H256.
     */
    getAssetAddress(index: number): H256;
    /**
     * Convert to an AssetTransferTransaction JSON object.
     * @returns An AssetTransferTransaction JSON object.
     */
    toJSON(): {
        type: string;
        data: {
            networkId: string;
            burns: {
                prevOut: {
                    transactionHash: string;
                    index: number;
                    assetType: string;
                    amount: string | number;
                };
                timelock: import("./AssetTransferInput").Timelock | null;
                lockScript: number[];
                unlockScript: number[];
            }[];
            inputs: {
                prevOut: {
                    transactionHash: string;
                    index: number;
                    assetType: string;
                    amount: string | number;
                };
                timelock: import("./AssetTransferInput").Timelock | null;
                lockScript: number[];
                unlockScript: number[];
            }[];
            outputs: {
                lockScriptHash: string;
                parameters: number[][];
                assetType: string;
                amount: string | number;
            }[];
        };
    };
}
