import { AssetTransferAddress, H256, PlatformAddress } from "codechain-primitives";
import { AssetMintTransaction } from "./transaction/AssetMintTransaction";
import { NetworkId } from "./types";
import { U256 } from "./U256";
/**
 * Object that contains information about the Asset when performing AssetMintTransaction.
 */
export declare class AssetScheme {
    static fromJSON(data: any): AssetScheme;
    readonly networkId?: NetworkId;
    readonly shardId?: number;
    readonly metadata: string;
    readonly amount: U256;
    readonly registrar: PlatformAddress | null;
    readonly pool: {
        assetType: H256;
        amount: U256;
    }[];
    constructor(data: {
        networkId?: NetworkId;
        shardId?: number;
        metadata: string;
        amount: U256;
        registrar: PlatformAddress | null;
        pool: {
            assetType: H256;
            amount: U256;
        }[];
    });
    toJSON(): {
        metadata: string;
        amount: string | number;
        registrar: string | null;
        pool: {
            assetType: string;
            amount: string | number;
        }[];
    };
    createMintTransaction(params: {
        recipient: AssetTransferAddress | string;
    }): AssetMintTransaction;
}
