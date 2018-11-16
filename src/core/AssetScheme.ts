import {
    AssetTransferAddress,
    H256,
    PlatformAddress
} from "codechain-primitives";

import { AssetMintOutput } from "./transaction/AssetMintOutput";
import { AssetMintTransaction } from "./transaction/AssetMintTransaction";
import { NetworkId } from "./types";
import { U64 } from "./U64";

export interface AssetSchemeJSON {
    metadata: string;
    amount: string;
    registrar: string | null;
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
        const { metadata, amount, registrar, pool } = data;
        return new AssetScheme({
            metadata,
            amount: U64.ensure(amount),
            registrar:
                registrar === null ? null : PlatformAddress.ensure(registrar),
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
    public readonly registrar: PlatformAddress | null;
    public readonly pool: { assetType: H256; amount: U64 }[];

    constructor(data: {
        networkId?: NetworkId;
        shardId?: number;
        metadata: string;
        amount: U64;
        registrar: PlatformAddress | null;
        pool: { assetType: H256; amount: U64 }[];
    }) {
        this.networkId = data.networkId;
        this.shardId = data.shardId;
        this.metadata = data.metadata;
        this.registrar = data.registrar;
        this.amount = data.amount;
        this.pool = data.pool;
    }

    public toJSON(): AssetSchemeJSON {
        const { metadata, amount, registrar, pool } = this;
        return {
            metadata,
            amount: `0x${amount.toString(16)}`,
            registrar: registrar === null ? null : registrar.toString(),
            pool: pool.map(a => ({
                assetType: a.assetType.value,
                amount: `0x${a.amount.toString(16)}`
            }))
        };
    }

    public createMintTransaction(params: {
        recipient: AssetTransferAddress | string;
    }): AssetMintTransaction {
        const { recipient } = params;
        const { networkId, shardId, metadata, amount, registrar } = this;
        if (networkId === undefined) {
            throw Error(`networkId is undefined`);
        }
        if (shardId === undefined) {
            throw Error(`shardId is undefined`);
        }
        return new AssetMintTransaction({
            networkId,
            shardId,
            metadata,
            output: new AssetMintOutput({
                amount,
                recipient: AssetTransferAddress.ensure(recipient)
            }),
            registrar
        });
    }
}
