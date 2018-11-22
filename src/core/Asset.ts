import { Buffer } from "buffer";
import { AssetTransferAddress, H160 } from "codechain-primitives";

import { H256 } from "./H256";
import { AssetOutPoint } from "./transaction/AssetOutPoint";
import { AssetTransferInput, Timelock } from "./transaction/AssetTransferInput";
import { AssetTransferOutput } from "./transaction/AssetTransferOutput";
import { AssetTransferTransaction } from "./transaction/AssetTransferTransaction";
import { NetworkId } from "./types";
import { U64 } from "./U64";

export interface AssetJSON {
    assetType: string;
    lockScriptHash: string;
    parameters: number[][];
    amount: string;
    orderHash: string | null;
    // The `hash` and the `index` are not included in an RPC response. See
    // getAsset() in chain.ts for more details.
    transactionHash: string;
    transactionOutputIndex: number;
}

export interface AssetData {
    assetType: H256;
    lockScriptHash: H160;
    parameters: Buffer[];
    amount: U64;
    orderHash?: H256 | null;
    transactionHash: H256;
    transactionOutputIndex: number;
}
/**
 * Object created as an AssetMintTransaction or AssetTransferTransaction.
 */
export class Asset {
    public static fromJSON(data: AssetJSON) {
        const {
            assetType,
            lockScriptHash,
            parameters,
            amount,
            orderHash,
            transactionHash,
            transactionOutputIndex
        } = data;
        return new Asset({
            assetType: new H256(assetType),
            lockScriptHash: new H160(lockScriptHash),
            parameters: parameters.map((p: Buffer | number[]) =>
                Buffer.from(p)
            ),
            amount: U64.ensure(amount),
            orderHash: orderHash === null ? orderHash : H256.ensure(orderHash),
            transactionHash: new H256(transactionHash),
            transactionOutputIndex
        });
    }

    public readonly assetType: H256;
    public readonly lockScriptHash: H160;
    public readonly parameters: Buffer[];
    public readonly amount: U64;
    public readonly outPoint: AssetOutPoint;
    public readonly orderHash: H256 | null;

    constructor(data: AssetData) {
        const {
            transactionHash,
            transactionOutputIndex,
            assetType,
            amount,
            orderHash = null,
            lockScriptHash,
            parameters
        } = data;
        this.assetType = data.assetType;
        this.lockScriptHash = data.lockScriptHash;
        this.parameters = data.parameters;
        this.amount = data.amount;
        this.orderHash = orderHash;
        this.outPoint = new AssetOutPoint({
            transactionHash,
            index: transactionOutputIndex,
            assetType,
            amount,
            lockScriptHash,
            parameters
        });
    }

    public toJSON(): AssetJSON {
        const {
            assetType,
            lockScriptHash,
            parameters,
            orderHash,
            amount,
            outPoint
        } = this;
        const { transactionHash, index } = outPoint;
        return {
            assetType: assetType.value,
            lockScriptHash: lockScriptHash.value,
            parameters: parameters.map(p => [...p]),
            amount: `0x${amount.toString(16)}`,
            orderHash: orderHash === null ? null : orderHash.toString(),
            transactionHash: transactionHash.value,
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
            address: AssetTransferAddress | string;
            amount: U64;
        }>;
        timelock?: null | Timelock;
        networkId: NetworkId;
    }): AssetTransferTransaction {
        const { outPoint, assetType } = this;
        const { recipients = [], timelock = null, networkId } = params;

        return new AssetTransferTransaction({
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
                        amount: recipient.amount
                    })
            ),
            orders: [],
            networkId
        });
    }
}
