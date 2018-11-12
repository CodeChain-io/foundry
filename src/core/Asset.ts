import { Buffer } from "buffer";
import { AssetTransferAddress, H160 } from "codechain-primitives";

import { H256 } from "./H256";
import { AssetOutPoint } from "./transaction/AssetOutPoint";
import { AssetTransferInput, Timelock } from "./transaction/AssetTransferInput";
import { AssetTransferOutput } from "./transaction/AssetTransferOutput";
import { AssetTransferTransaction } from "./transaction/AssetTransferTransaction";
import { NetworkId } from "./types";
import { U256 } from "./U256";

export interface AssetJSON {
    assetType: string;
    lockScriptHash: string;
    parameters: number[][];
    amount: string;
    // The `hash` and the `index` are not included in an RPC response. See
    // getAsset() in chain.ts for more details.
    transactionHash: string;
    transactionOutputIndex: number;
}

export interface AssetData {
    assetType: H256;
    lockScriptHash: H160;
    parameters: Buffer[];
    amount: U256;
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
            transactionHash,
            transactionOutputIndex
        } = data;
        return new Asset({
            assetType: new H256(assetType),
            lockScriptHash: new H160(lockScriptHash),
            parameters: parameters.map((p: Buffer | number[]) =>
                Buffer.from(p)
            ),
            amount: U256.ensure(amount),
            transactionHash: new H256(transactionHash),
            transactionOutputIndex
        });
    }

    public readonly assetType: H256;
    public readonly lockScriptHash: H160;
    public readonly parameters: Buffer[];
    public readonly amount: U256;
    public readonly outPoint: AssetOutPoint;

    constructor(data: AssetData) {
        const {
            transactionHash,
            transactionOutputIndex,
            assetType,
            amount,
            lockScriptHash,
            parameters
        } = data;
        this.assetType = data.assetType;
        this.lockScriptHash = data.lockScriptHash;
        this.parameters = data.parameters;
        this.amount = data.amount;
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
            amount,
            outPoint
        } = this;
        const { transactionHash, index } = outPoint;
        return {
            assetType: assetType.value,
            lockScriptHash: lockScriptHash.value,
            parameters: parameters.map(p => [...p]),
            amount: `0x${amount.toString(16)}`,
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
            amount: U256;
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
            networkId
        });
    }
}
