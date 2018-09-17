import { Buffer } from "buffer";
import { AssetTransferAddress } from "codechain-primitives";

import { H256 } from "./H256";
import { AssetOutPoint } from "./transaction/AssetOutPoint";
import { AssetTransferInput } from "./transaction/AssetTransferInput";
import { AssetTransferOutput } from "./transaction/AssetTransferOutput";
import { AssetTransferTransaction } from "./transaction/AssetTransferTransaction";
import { NetworkId } from "./types";

export interface AssetData {
    assetType: H256;
    lockScriptHash: H256;
    parameters: Buffer[];
    amount: number;
    transactionHash: H256;
    transactionOutputIndex: number;
}
/**
 * Object created as an AssetMintTransaction or AssetTransferTransaction.
 */
export class Asset {
    public static fromJSON(data: any) {
        // FIXME: use camelCase for all
        const {
            asset_type,
            lock_script_hash,
            parameters,
            amount,
            transactionHash,
            transactionOutputIndex
        } = data;
        return new Asset({
            assetType: new H256(asset_type),
            lockScriptHash: new H256(lock_script_hash),
            parameters: parameters.map((p: Buffer | number[]) =>
                Buffer.from(p)
            ),
            amount,
            transactionHash: new H256(transactionHash),
            transactionOutputIndex
        });
    }
    public assetType: H256;
    public lockScriptHash: H256;
    public parameters: Buffer[];
    public amount: number;
    public outPoint: AssetOutPoint;

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

    public toJSON() {
        const {
            assetType,
            lockScriptHash,
            parameters,
            amount,
            outPoint
        } = this;
        const { transactionHash, index } = outPoint;
        return {
            asset_type: assetType.value,
            lock_script_hash: lockScriptHash.value,
            parameters,
            amount,
            transactionHash: transactionHash.value,
            transactionOutputIndex: index
        };
    }

    public createTransferInput(): AssetTransferInput {
        return new AssetTransferInput({
            prevOut: this.outPoint
        });
    }

    public createTransferTransaction(params: {
        recipients?: Array<{
            address: AssetTransferAddress | string;
            amount: number;
        }>;
        nonce?: number;
        networkId: NetworkId;
    }): AssetTransferTransaction {
        const { outPoint, assetType } = this;
        const { recipients = [], nonce = 0, networkId } = params;

        return new AssetTransferTransaction({
            burns: [],
            inputs: [
                new AssetTransferInput({
                    prevOut: outPoint,
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
            networkId,
            nonce
        });
    }
}
