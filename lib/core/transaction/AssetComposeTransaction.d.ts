/// <reference types="node" />
import { H256, PlatformAddress } from "codechain-primitives/lib";
import { SignatureTag } from "../../utils";
import { Asset } from "../Asset";
import { AssetScheme } from "../AssetScheme";
import { NetworkId } from "../types";
import { AssetMintOutput } from "./AssetMintOutput";
import { AssetTransferInput } from "./AssetTransferInput";
/**
 * Compose assets.
 */
export declare class AssetComposeTransaction {
    /**
     * Create an AssetComposeTransaction from an AssetComposeTransaction JSON object.
     * @param obj An AssetComposeTransaction JSON object.
     * @returns An AssetComposeTransaction.
     */
    static fromJSON(obj: any): AssetComposeTransaction;
    readonly networkId: NetworkId;
    readonly shardId: number;
    readonly metadata: string;
    readonly registrar: PlatformAddress | null;
    readonly inputs: AssetTransferInput[];
    readonly output: AssetMintOutput;
    readonly type: string;
    /**
     * @param params.networkId A network ID of the transaction.
     * @param params.shardId A shard ID of the transaction.
     * @param params.metadata A metadata of the asset.
     * @param params.registrar A registrar of the asset.
     * @param params.inputs A list of inputs of the transaction.
     * @param params.output An output of the transaction.
     */
    constructor(params: {
        networkId: NetworkId;
        shardId: number;
        metadata: string;
        registrar: PlatformAddress | null;
        inputs: AssetTransferInput[];
        output: AssetMintOutput;
    });
    /**
     * Convert to an AssetComposeTransaction JSON object.
     * @returns An AssetComposeTransaction JSON object.
     */
    toJSON(): {
        type: string;
        data: {
            networkId: string;
            shardId: number;
            metadata: string;
            registrar: PlatformAddress | null;
            output: {
                lockScriptHash: string;
                parameters: number[][];
                amount: string | number | undefined;
            };
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
        };
    };
    /**
     * Convert to an object for RLP encoding.
     */
    toEncodeObject(): (string | number | (string | number)[] | Buffer[] | ((string | number)[] | Buffer | number[][])[][])[];
    /**
     * Convert to RLP bytes.
     */
    rlpBytes(): Buffer;
    /**
     * Get the hash of an AssetComposeTransaction.
     * @returns A transaction hash.
     */
    hash(): H256;
    /**
     * Get a hash of the transaction that doesn't contain the scripts. The hash
     * is used as a message to create a signature for a transaction.
     * @returns A hash.
     */
    hashWithoutScript(params?: {
        tag: SignatureTag;
        index: number;
    }): H256;
    /**
     * Add an AssetTransferInput to spend.
     * @param inputs An array of either an AssetTransferInput or an Asset.
     * @returns The modified AssetComposeTransaction.
     */
    addInputs(inputs: AssetTransferInput | Asset | Array<AssetTransferInput | Asset>, ...rest: Array<AssetTransferInput | Asset>): AssetComposeTransaction;
    /**
     * Get the output of this transaction.
     * @returns An Asset.
     */
    getComposedAsset(): Asset;
    /**
     * Get the asset scheme of this transaction.
     * @return An AssetScheme.
     */
    getAssetScheme(): AssetScheme;
    /**
     * Get the address of the asset scheme. An asset scheme address equals to an
     * asset type value.
     * @returns An asset scheme address which is H256.
     */
    getAssetSchemeAddress(): H256;
    /**
     * Get the asset address of the output.
     * @returns An asset address which is H256.
     */
    getAssetAddress(): H256;
}
