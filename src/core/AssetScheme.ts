// FIXME: Use interface instead of importing key class.
import { AssetTransferAddress } from "../key/AssetTransferAddress";
import { PlatformAddress } from "../key/classes";

import { AssetMintTransaction } from "./transaction/AssetMintTransaction";

type NetworkId = string;

export type AssetSchemeData = {
    networkId: NetworkId;
    shardId: number;
    worldId: number;
    metadata: string;
    amount: number;
    registrar: PlatformAddress | null;
};
/**
 * Object that contains information about the Asset when performing AssetMintTransaction.
 */
export class AssetScheme {
    networkId: NetworkId;
    shardId: number;
    worldId: number;
    metadata: string;
    amount: number;
    registrar: PlatformAddress | null;

    constructor(data: AssetSchemeData) {
        this.networkId = data.networkId;
        this.shardId = data.shardId;
        this.worldId = data.worldId;
        this.metadata = data.metadata;
        this.registrar = data.registrar;
        this.amount = data.amount;
    }

    static fromJSON(data: any) {
        return new AssetScheme(data);
    }

    toJSON() {
        const { networkId, metadata, amount, registrar } = this;
        return {
            networkId,
            metadata,
            amount,
            registrar: registrar === null ? null : registrar.toString()
        };
    }

    createMintTransaction(params: { recipient: AssetTransferAddress | string, nonce?: number }): AssetMintTransaction {
        const { recipient, nonce = 0 } = params;
        const { networkId, shardId, worldId, metadata, amount, registrar } = this;
        return new AssetMintTransaction({
            networkId,
            shardId,
            worldId,
            metadata,
            output: {
                amount,
                ...AssetTransferAddress.ensure(recipient).getLockScriptHashAndParameters(),
            },
            registrar,
            nonce,
        });
    }
}
