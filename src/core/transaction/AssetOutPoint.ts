import { H256 } from "../H256";

export interface AssetOutPointData {
    transactionHash: H256;
    index: number;
    assetType: H256;
    amount: number;
    lockScriptHash?: H256;
    parameters?: Buffer[];
}

/**
 * AssetOutPoint consists of transactionHash and index, asset type, and amount.
 *
 * - The transaction that it points to must be either AssetMint or AssetTransfer.
 * - Index is what decides which Asset to point to amongst the Asset list that transaction creates.
 * - The asset type and amount must be identical to the Asset that it points to.
 */
export class AssetOutPoint {
    /**
     * Create an AssetOutPoint from an AssetOutPoint JSON object.
     * @param data An AssetOutPoint JSON object.
     * @returns An AssetOutPoint.
     */
    public static fromJSON(data: any) {
        const { transactionHash, index, assetType, amount } = data;
        return new this({
            transactionHash: new H256(transactionHash),
            index,
            assetType: new H256(assetType),
            amount
        });
    }
    public readonly transactionHash: H256;
    public readonly index: number;
    public readonly assetType: H256;
    public readonly amount: number;
    public readonly lockScriptHash?: H256;
    public readonly parameters?: Buffer[];

    /**
     * @param data.transactionHash A transaction hash where the Asset is created.
     * @param data.index The index in the output of the transaction.
     * @param data.assetType The asset type of the asset that it points to.
     * @param data.amount The asset amount of the asset that it points to.
     * @param data.lockScriptHash The lock script hash of the asset.
     * @param data.parameters The parameters of the asset.
     */
    constructor(data: AssetOutPointData) {
        const {
            transactionHash,
            index,
            assetType,
            amount,
            lockScriptHash,
            parameters
        } = data;
        this.transactionHash = transactionHash;
        this.index = index;
        this.assetType = assetType;
        this.amount = amount;
        this.lockScriptHash = lockScriptHash;
        this.parameters = parameters;
    }

    /**
     * Convert to an object for RLP encoding.
     */
    public toEncodeObject() {
        const { transactionHash, index, assetType, amount } = this;
        return [
            transactionHash.toEncodeObject(),
            index,
            assetType.toEncodeObject(),
            amount
        ];
    }

    /**
     * Convert to an AssetOutPoint JSON object.
     * @returns An AssetOutPoint JSON object.
     */
    public toJSON() {
        const { transactionHash, index, assetType, amount } = this;
        return {
            transactionHash: transactionHash.value,
            index,
            assetType: assetType.value,
            amount
        };
    }
}
