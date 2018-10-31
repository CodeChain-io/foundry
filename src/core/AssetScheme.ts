import {
    AssetTransferAddress,
    H256,
    PlatformAddress
} from "codechain-primitives";

import { AssetMintOutput } from "./transaction/AssetMintOutput";
import { AssetMintTransaction } from "./transaction/AssetMintTransaction";
import { NetworkId } from "./types";

/**
 * Object that contains information about the Asset when performing AssetMintTransaction.
 */
export class AssetScheme {
    public static fromJSON(data: any) {
        const { metadata, amount, registrar, pool } = data;
        return new AssetScheme({
            metadata,
            amount,
            registrar:
                registrar === null ? null : PlatformAddress.ensure(registrar),
            pool: pool.map(({ assetType, amount: assetAmount }: any) => ({
                assetType: H256.ensure(assetType),
                amount: assetAmount
            }))
        });
    }

    public readonly networkId?: NetworkId;
    public readonly shardId?: number;
    public readonly metadata: string;
    public readonly amount: number;
    public readonly registrar: PlatformAddress | null;
    public readonly pool: { assetType: H256; amount: number }[];

    constructor(data: {
        networkId?: NetworkId;
        shardId?: number;
        metadata: string;
        amount: number;
        registrar: PlatformAddress | null;
        pool: { assetType: H256; amount: number }[];
    }) {
        this.networkId = data.networkId;
        this.shardId = data.shardId;
        this.metadata = data.metadata;
        this.registrar = data.registrar;
        this.amount = data.amount;
        this.pool = data.pool;
    }

    public toJSON() {
        const { metadata, amount, registrar, pool } = this;
        return {
            metadata,
            amount,
            registrar: registrar === null ? null : registrar.toString(),
            pool: pool.map(a => ({
                assetType: a.assetType.value,
                amount: a.amount
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
