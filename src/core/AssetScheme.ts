import { AssetTransferAddress, PlatformAddress } from "codechain-primitives";

import { AssetMintTransaction } from "./transaction/AssetMintTransaction";
import { NetworkId } from "./types";

export interface AssetSchemeData {
    networkId: NetworkId;
    shardId: number;
    worldId: number;
    metadata: string;
    amount: number;
    registrar: PlatformAddress | null;
}
/**
 * Object that contains information about the Asset when performing AssetMintTransaction.
 */
export class AssetScheme {
    public static fromJSON(data: any) {
        return new AssetScheme(data);
    }
    public networkId: NetworkId;
    public shardId: number;
    public worldId: number;
    public metadata: string;
    public amount: number;
    public registrar: PlatformAddress | null;

    constructor(data: AssetSchemeData) {
        this.networkId = data.networkId;
        this.shardId = data.shardId;
        this.worldId = data.worldId;
        this.metadata = data.metadata;
        this.registrar = data.registrar;
        this.amount = data.amount;
    }

    public toJSON() {
        const { networkId, metadata, amount, registrar } = this;
        return {
            networkId,
            metadata,
            amount,
            registrar: registrar === null ? null : registrar.toString()
        };
    }

    public createMintTransaction(params: {
        recipient: AssetTransferAddress | string;
        nonce?: number;
    }): AssetMintTransaction {
        const { recipient, nonce = 0 } = params;
        const {
            networkId,
            shardId,
            worldId,
            metadata,
            amount,
            registrar
        } = this;
        return new AssetMintTransaction({
            networkId,
            shardId,
            worldId,
            metadata,
            output: {
                amount,
                recipient: AssetTransferAddress.ensure(recipient)
            },
            registrar,
            nonce
        });
    }
}
