import {
    AssetTransferAddress,
    AssetTransferAddressValue,
    H160,
    PlatformAddress,
    U64
} from "codechain-primitives";

import { AssetMintOutput } from "./transaction/AssetMintOutput";
import { MintAsset } from "./transaction/MintAsset";
import { NetworkId } from "./types";

export interface AssetSchemeJSON {
    metadata: string;
    supply: string;
    approver: string | null;
    registrar: string | null;
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
            registrar,
            allowedScriptHashes,
            pool
        } = data;
        return new AssetScheme({
            metadata,
            supply: U64.ensure(supply),
            approver:
                approver == null ? null : PlatformAddress.ensure(approver),
            registrar:
                registrar == null ? null : PlatformAddress.ensure(registrar),
            allowedScriptHashes:
                allowedScriptHashes == null
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
    public readonly registrar: PlatformAddress | null;
    public readonly allowedScriptHashes: H160[];
    public readonly pool: { assetType: H160; quantity: U64 }[];

    constructor(data: {
        networkId?: NetworkId;
        shardId?: number;
        metadata: string | object;
        supply: U64;
        approver: PlatformAddress | null;
        registrar: PlatformAddress | null;
        allowedScriptHashes: H160[];
        pool: { assetType: H160; quantity: U64 }[];
    }) {
        this.networkId = data.networkId;
        this.shardId = data.shardId;
        this.metadata =
            typeof data.metadata === "string"
                ? data.metadata
                : JSON.stringify(data.metadata);
        this.approver = data.approver;
        this.registrar = data.registrar;
        this.allowedScriptHashes = data.allowedScriptHashes;
        this.supply = data.supply;
        this.pool = data.pool;
    }

    public toJSON(): AssetSchemeJSON {
        const {
            metadata,
            supply,
            approver,
            registrar,
            allowedScriptHashes,
            pool
        } = this;
        return {
            metadata,
            supply: supply.toJSON(),
            approver: approver == null ? null : approver.toString(),
            registrar: registrar == null ? null : registrar.toString(),
            allowedScriptHashes: allowedScriptHashes.map(hash => hash.toJSON()),
            pool: pool.map(a => ({
                assetType: a.assetType.toJSON(),
                quantity: a.quantity.toJSON()
            }))
        };
    }

    public createMintTransaction(params: {
        recipient: AssetTransferAddressValue;
    }): MintAsset {
        const { recipient } = params;
        const {
            networkId,
            shardId,
            metadata,
            supply,
            approver,
            registrar,
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
            registrar,
            allowedScriptHashes,
            approvals: []
        });
    }
}
