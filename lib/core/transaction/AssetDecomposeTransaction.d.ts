/// <reference types="node" />
import { H256 } from "codechain-primitives/lib";
import { Asset } from "../Asset";
import { AssetTransferOutputValue, NetworkId } from "../types";
import { AssetTransferInput } from "./AssetTransferInput";
import { AssetTransferOutput } from "./AssetTransferOutput";
/**
 * Decompose assets. The sum of inputs must be whole supply of the asset.
 */
export declare class AssetDecomposeTransaction {
    /**
     * Create an AssetDecomposeTransaction from an AssetDecomposeTransaction JSON object.
     * @param obj An AssetDecomposeTransaction JSON object.
     * @returns An AssetDecomposeTransaction.
     */
    static fromJSON(obj: any): AssetDecomposeTransaction;
    readonly input: AssetTransferInput;
    readonly outputs: AssetTransferOutput[];
    readonly networkId: NetworkId;
    readonly type: string;
    /**
     * @param params.inputs An array of AssetTransferInput to decompose.
     * @param params.outputs An array of AssetTransferOutput to create.
     * @param params.networkId A network ID of the transaction.
     */
    constructor(params: {
        input: AssetTransferInput;
        outputs: AssetTransferOutput[];
        networkId: NetworkId;
    });
    /**
     * Convert to an AssetDecomposeTransaction JSON object.
     * @returns An AssetDecomposeTransaction JSON object.
     */
    toJSON(): {
        type: string;
        data: {
            input: {
                prevOut: {
                    transactionHash: string;
                    index: number;
                    assetType: string;
                    amount: string | number;
                };
                timelock: import("./AssetTransferInput").Timelock | null;
                lockScript: number[];
                unlockScript: number[];
            };
            outputs: {
                lockScriptHash: string;
                parameters: number[][];
                assetType: string;
                amount: string | number;
            }[];
            networkId: string;
        };
    };
    /**
     * Convert to an object for RLP encoding.
     */
    toEncodeObject(): (string | number | ((string | number)[] | Buffer | number[][])[] | (string | number | Buffer[])[][])[];
    /**
     * Convert to RLP bytes.
     */
    rlpBytes(): Buffer;
    /**
     * Get the hash of an AssetDecomposeTransaction.
     * @returns A transaction hash.
     */
    hash(): H256;
    /**
     * Get a hash of the transaction that doesn't contain the scripts. The hash
     * is used as a message to create a signature for a transaction.
     * @returns A hash.
     */
    hashWithoutScript(): H256;
    /**
     * Add AssetTransferOutputs to create.
     * @param outputs An array of either an AssetTransferOutput or an object
     * containing amount, asset type, and recipient.
     */
    addOutputs(outputs: AssetTransferOutputValue | Array<AssetTransferOutputValue>, ...rest: Array<AssetTransferOutputValue>): void;
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
     * Get the asset address of an output.
     * @param index An index indicating the output.
     * @returns An asset address which is H256.
     */
    getAssetAddress(index: number): H256;
}
