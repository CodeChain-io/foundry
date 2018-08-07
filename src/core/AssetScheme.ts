// FIXME: Use interface instead of importing key class.
import { AssetTransferAddress } from "../key/AssetTransferAddress";
import { PlatformAddress } from "../key/classes";

import { H160 } from "./H160";
import { AssetMintTransaction } from "./transaction/AssetMintTransaction";

export type AssetSchemeData = {
    networkId: number;
    shardId: number;
    worldId: number;
    metadata: string;
    amount: number;
    registrar: PlatformAddress | H160 | string | null;
};
/**
 * Object that contains information about the Asset when performing AssetMintTransaction.
 */
export class AssetScheme {
    networkId: number;
    shardId: number;
    worldId: number;
    metadata: string;
    amount: number;
    registrar: H160 | null;

    constructor(data: AssetSchemeData) {
        this.networkId = data.networkId;
        this.shardId = data.shardId;
        this.worldId = data.worldId;
        this.metadata = data.metadata;
        this.registrar = data.registrar === null ? null : PlatformAddress.ensureAccount(data.registrar);
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
            registrar: registrar === null ? null : registrar.value
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
