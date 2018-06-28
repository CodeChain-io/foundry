import { H256 } from "./H256";
import { AssetOutPoint } from "./transaction/AssetTransferTransaction";

export type AssetData = {
    assetType: H256;
    lockScriptHash: H256;
    parameters: Buffer[];
    amount: number;
    transactionHash: H256;
    transactionOutputIndex: number;
};
/**
 * Object created as an AssetMintTransaction or AssetTransferTransaction.
 */
export class Asset {
    assetType: H256;
    lockScriptHash: H256;
    parameters: Buffer[];
    amount: number;
    outPoint: AssetOutPoint;

    constructor(data: AssetData) {
        const { transactionHash, transactionOutputIndex, assetType, amount } = data;
        this.assetType = data.assetType;
        this.lockScriptHash = data.lockScriptHash;
        this.parameters = data.parameters;
        this.amount = data.amount;
        this.outPoint = new AssetOutPoint({
            transactionHash,
            index: transactionOutputIndex,
            assetType,
            amount,
        });
    }

    static fromJSON(data: any) {
        // FIXME: use camelCase for all
        const { asset_type, lock_script_hash, parameters, amount, transactionHash, transactionOutputIndex } = data;
        return new Asset({
            assetType: new H256(asset_type),
            lockScriptHash: new H256(lock_script_hash),
            parameters,
            amount,
            transactionHash: new H256(transactionHash),
            transactionOutputIndex,
        });
    }

    toJSON() {
        const { assetType, lockScriptHash, parameters, amount, outPoint } = this;
        const { transactionHash, index } = outPoint.data;
        return {
            asset_type: assetType.value,
            lock_script_hash: lockScriptHash.value,
            parameters,
            amount,
            transactionHash: transactionHash.value,
            transactionOutputIndex: index,
        };
    }
}
