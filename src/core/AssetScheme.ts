import {
    AssetTransferAddress,
    H160,
    H256,
    PlatformAddress
} from "codechain-primitives";

import { AssetMintOutput } from "./transaction/AssetMintOutput";
import { MintAsset } from "./transaction/MintAsset";
import { NetworkId } from "./types";
import { U64 } from "./U64";

export interface AssetSchemeJSON {
    metadata: string;
    amount: string;
    approver: string | null;
    administrator: string | null;
    allowedScriptHashes: string[] | null;
    pool: {
        assetType: string;
        amount: string;
    }[];
}

/**
 * Object that contains information about the Asset when performing AssetMintTransaction.
 */
export class AssetScheme {
    public static fromJSON(data: AssetSchemeJSON) {
        const {
            metadata,
            amount,
            approver,
            administrator,
            allowedScriptHashes,
            pool
        } = data;
        return new AssetScheme({
            metadata,
            amount: U64.ensure(amount),
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
            pool: pool.map(({ assetType, amount: assetAmount }: any) => ({
                assetType: H256.ensure(assetType),
                amount: U64.ensure(assetAmount)
            }))
        });
    }

    public readonly networkId?: NetworkId;
    public readonly shardId?: number;
    public readonly metadata: string;
    public readonly amount: U64;
    public readonly approver: PlatformAddress | null;
    public readonly administrator: PlatformAddress | null;
    public readonly allowedScriptHashes: H160[];
    public readonly pool: { assetType: H256; amount: U64 }[];

    constructor(data: {
        networkId?: NetworkId;
        shardId?: number;
        metadata: string;
        amount: U64;
        approver: PlatformAddress | null;
        administrator: PlatformAddress | null;
        allowedScriptHashes: H160[];
        pool: { assetType: H256; amount: U64 }[];
    }) {
        this.networkId = data.networkId;
        this.shardId = data.shardId;
        this.metadata = data.metadata;
        this.approver = data.approver;
        this.administrator = data.administrator;
        this.allowedScriptHashes = data.allowedScriptHashes;
        this.amount = data.amount;
        this.pool = data.pool;
    }

    public toJSON(): AssetSchemeJSON {
        const {
            metadata,
            amount,
            approver,
            administrator,
            allowedScriptHashes,
            pool
        } = this;
        return {
            metadata,
            amount: amount.toJSON(),
            approver: approver === null ? null : approver.toString(),
            administrator:
                administrator === null ? null : administrator.toString(),
            allowedScriptHashes: allowedScriptHashes.map(hash => hash.toJSON()),
            pool: pool.map(a => ({
                assetType: a.assetType.toJSON(),
                amount: a.amount.toJSON()
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
            amount,
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
                amount,
                recipient: AssetTransferAddress.ensure(recipient)
            }),
            approver,
            administrator,
            allowedScriptHashes,
            approvals: []
        });
    }
}
