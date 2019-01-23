import {
    AssetTransferAddress,
    H160,
    PlatformAddress
} from "codechain-primitives";

import { AssetMintOutput } from "./transaction/AssetMintOutput";
import { MintAsset } from "./transaction/MintAsset";
import { NetworkId } from "./types";
import { U64 } from "./U64";

export interface AssetSchemeJSON {
    metadata: string;
    supply: string;
    approver: string | null;
    administrator: string | null;
    allowedScriptHashes: string[] | null;
    pool: {
        assetType: string;
        quantity: string;
    }[];
}

/**
 * Object that contains information about the Asset when performing AssetMintTransaction.
 */
export class AssetScheme {
    public static fromJSON(data: AssetSchemeJSON) {
        const {
            metadata,
            supply,
            approver,
            administrator,
            allowedScriptHashes,
            pool
        } = data;
        return new AssetScheme({
            metadata,
            supply: U64.ensure(supply),
            approver:
                approver === null ? null : PlatformAddress.ensure(approver),
            administrator:
                administrator === null
                    ? null
                    : PlatformAddress.ensure(administrator),
            allowedScriptHashes:
                allowedScriptHashes === null
                    ? []
                    : allowedScriptHashes.map((hash: string) =>
                          H160.ensure(hash)
                      ),
            pool: pool.map(({ assetType, quantity: assetQuantity }: any) => ({
                assetType: H160.ensure(assetType),
                quantity: U64.ensure(assetQuantity)
            }))
        });
    }

    public readonly networkId?: NetworkId;
    public readonly shardId?: number;
    public readonly metadata: string;
    public readonly supply: U64;
    public readonly approver: PlatformAddress | null;
    public readonly administrator: PlatformAddress | null;
    public readonly allowedScriptHashes: H160[];
    public readonly pool: { assetType: H160; quantity: U64 }[];

    constructor(data: {
        networkId?: NetworkId;
        shardId?: number;
        metadata: string;
        supply: U64;
        approver: PlatformAddress | null;
        administrator: PlatformAddress | null;
        allowedScriptHashes: H160[];
        pool: { assetType: H160; quantity: U64 }[];
    }) {
        this.networkId = data.networkId;
        this.shardId = data.shardId;
        this.metadata = data.metadata;
        this.approver = data.approver;
        this.administrator = data.administrator;
        this.allowedScriptHashes = data.allowedScriptHashes;
        this.supply = data.supply;
        this.pool = data.pool;
    }

    public toJSON(): AssetSchemeJSON {
        const {
            metadata,
            supply,
            approver,
            administrator,
            allowedScriptHashes,
            pool
        } = this;
        return {
            metadata,
            supply: supply.toJSON(),
            approver: approver === null ? null : approver.toString(),
            administrator:
                administrator === null ? null : administrator.toString(),
            allowedScriptHashes: allowedScriptHashes.map(hash => hash.toJSON()),
            pool: pool.map(a => ({
                assetType: a.assetType.toJSON(),
                quantity: a.quantity.toJSON()
            }))
        };
    }

    public createMintTransaction(params: {
        recipient: AssetTransferAddress | string;
    }): MintAsset {
        const { recipient } = params;
        const {
            networkId,
            shardId,
            metadata,
            supply,
            approver,
            administrator,
            allowedScriptHashes
        } = this;
        if (networkId === undefined) {
            throw Error(`networkId is undefined`);
        }
        if (shardId === undefined) {
            throw Error(`shardId is undefined`);
        }
        return new MintAsset({
            networkId,
            shardId,
            metadata,
            output: new AssetMintOutput({
                supply,
                recipient: AssetTransferAddress.ensure(recipient)
            }),
            approver,
            administrator,
            allowedScriptHashes,
            approvals: []
        });
    }
}
