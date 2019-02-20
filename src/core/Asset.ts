import { Buffer } from "buffer";
import {
    AssetTransferAddress,
    AssetTransferAddressValue,
    H160,
    H256,
    U64
} from "codechain-primitives";

import { AssetOutPoint } from "./transaction/AssetOutPoint";
import { AssetTransferInput, Timelock } from "./transaction/AssetTransferInput";
import { AssetTransferOutput } from "./transaction/AssetTransferOutput";
import { TransferAsset } from "./transaction/TransferAsset";
import { NetworkId } from "./types";

export interface AssetJSON {
    assetType: string;
    lockScriptHash: string;
    parameters: string[];
    quantity: string;
    orderHash: string | null;
    // The `hash` and the `index` are not included in an RPC response. See
    // getAsset() in chain.ts for more details.
    shardId: number;
    tracker: string;
    transactionOutputIndex: number;
}

export interface AssetData {
    assetType: H160;
    shardId: number;
    lockScriptHash: H160;
    parameters: Buffer[];
    quantity: U64;
    orderHash?: H256 | null;
    tracker: H256;
    transactionOutputIndex: number;
}
/**
 * Object created as an AssetMintTransaction or TransferAsset.
 */
export class Asset {
    public static fromJSON(data: AssetJSON) {
        const {
            assetType,
            shardId,
            lockScriptHash,
            parameters,
            quantity,
            orderHash,
            tracker,
            transactionOutputIndex
        } = data;
        return new Asset({
            assetType: new H160(assetType),
            shardId,
            lockScriptHash: new H160(lockScriptHash),
            parameters: parameters.map((p: string) => Buffer.from(p, "hex")),
            quantity: U64.ensure(quantity),
            orderHash: orderHash == null ? orderHash : H256.ensure(orderHash),
            tracker: new H256(tracker),
            transactionOutputIndex
        });
    }

    public readonly assetType: H160;
    public readonly shardId: number;
    public readonly lockScriptHash: H160;
    public readonly parameters: Buffer[];
    public readonly quantity: U64;
    public readonly outPoint: AssetOutPoint;
    public readonly orderHash: H256 | null;

    constructor(data: AssetData) {
        const {
            tracker,
            transactionOutputIndex,
            assetType,
            shardId,
            quantity,
            orderHash = null,
            lockScriptHash,
            parameters
        } = data;
        this.assetType = assetType;
        this.shardId = shardId;
        this.lockScriptHash = lockScriptHash;
        this.parameters = parameters;
        this.quantity = quantity;
        this.orderHash = orderHash;
        this.outPoint = new AssetOutPoint({
            tracker,
            index: transactionOutputIndex,
            assetType,
            shardId,
            quantity,
            lockScriptHash,
            parameters
        });
    }

    public toJSON(): AssetJSON {
        const {
            assetType,
            shardId,
            lockScriptHash,
            parameters,
            orderHash,
            quantity,
            outPoint
        } = this;
        const { tracker, index } = outPoint;
        return {
            assetType: assetType.toJSON(),
            shardId,
            lockScriptHash: lockScriptHash.toJSON(),
            parameters: parameters.map((p: Buffer) => p.toString("hex")),
            quantity: quantity.toJSON(),
            orderHash: orderHash == null ? null : orderHash.toJSON(),
            tracker: tracker.toJSON(),
            transactionOutputIndex: index
        };
    }

    public createTransferInput(options?: {
        timelock: Timelock | null;
    }): AssetTransferInput {
        const { timelock = null } = options || {};
        return new AssetTransferInput({
            prevOut: this.outPoint,
            timelock
        });
    }

    public createTransferTransaction(params: {
        recipients?: Array<{
            address: AssetTransferAddressValue;
            quantity: U64;
        }>;
        timelock?: null | Timelock;
        networkId: NetworkId;
        metadata?: string;
        approvals?: string[];
        expiration?: number;
    }): TransferAsset {
        const { outPoint, assetType, shardId } = this;
        const {
            recipients = [],
            timelock = null,
            networkId,
            metadata = "",
            approvals = [],
            expiration = null
        } = params;

        return new TransferAsset({
            burns: [],
            inputs: [
                new AssetTransferInput({
                    prevOut: outPoint,
                    timelock,
                    lockScript: Buffer.from([]),
                    unlockScript: Buffer.from([])
                })
            ],
            outputs: recipients.map(
                recipient =>
                    new AssetTransferOutput({
                        recipient: AssetTransferAddress.ensure(
                            recipient.address
                        ),
                        assetType,
                        shardId,
                        quantity: recipient.quantity
                    })
            ),
            orders: [],
            networkId,
            metadata,
            approvals,
            expiration
        });
    }
}
