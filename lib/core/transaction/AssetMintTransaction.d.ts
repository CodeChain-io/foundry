/// <reference types="node" />
import { PlatformAddress } from "codechain-primitives";
import { Asset } from "../Asset";
import { AssetScheme } from "../AssetScheme";
import { H256 } from "../H256";
import { NetworkId } from "../types";
import { AssetMintOutput } from "./AssetMintOutput";
/**
 * Creates a new asset type and that asset itself.
 *
 * The owner of the new asset created can be assigned by a lock script hash and parameters.
 *  - A metadata is a string that explains the asset's type.
 *  - Amount defines the quantity of asset to be created. If set as null, it
 *  will be set as the maximum value of a 64-bit unsigned integer by default.
 *  - If registrar exists, the registrar must be the Signer of the Parcel when
 *  sending the created asset through AssetTransferTransaction.
 */
export declare class AssetMintTransaction {
    /**
     * Create an AssetMintTransaction from an AssetMintTransaction JSON object.
     * @param data An AssetMintTransaction JSON object.
     * @returns An AssetMintTransaction.
     */
    static fromJSON(data: any): AssetMintTransaction;
    readonly networkId: NetworkId;
    readonly shardId: number;
    readonly metadata: string;
    readonly output: AssetMintOutput;
    readonly registrar: PlatformAddress | null;
    readonly type: string;
    /**
     * @param data.networkId A network ID of the transaction.
     * @param data.shardId A shard ID of the transaction.
     * @param data.metadata A metadata of the asset.
     * @param data.output.lockScriptHash A lock script hash of the output.
     * @param data.output.parameters Parameters of the output.
     * @param data.output.amount Asset amount of the output.
     * @param data.registrar A registrar of the asset.
     */
    constructor(data: {
        networkId: NetworkId;
        shardId: number;
        metadata: string;
        output: AssetMintOutput;
        registrar: PlatformAddress | null;
    });
    /**
     * Convert to an AssetMintTransaction JSON object.
     * @returns An AssetMintTransaction JSON object.
     */
    toJSON(): {
        type: string;
        data: {
            networkId: string;
            shardId: number;
            metadata: string;
            output: {
                lockScriptHash: string;
                parameters: number[][];
                amount: string | number | undefined;
            };
            registrar: string | null;
        };
    };
    /**
     * Convert to an object for RLP encoding.
     */
    toEncodeObject(): (string | number | (string | number)[] | Buffer[])[];
    /**
     * Convert to RLP bytes.
     */
    rlpBytes(): Buffer;
    /**
     * Get the hash of an AssetMintTransaction.
     * @returns A transaction hash.
     */
    hash(): H256;
    /**
     * Get the output of this transaction.
     * @returns An Asset.
     */
    getMintedAsset(): Asset;
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
