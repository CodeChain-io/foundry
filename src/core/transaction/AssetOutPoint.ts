import { H256 } from "../H256";

export type AssetOutPointData = {
    transactionHash: H256;
    index: number;
    assetType: H256;
    amount: number;
    lockScriptHash?: H256;
    parameters?: Buffer[];
};

/**
 * AssetOutPoint consists of transactionHash and index, asset type, and amount.
 *
 * - The transaction that it points to must be either AssetMint or AssetTransfer.
 * - Index is what decides which Asset to point to amongst the Asset list that transaction creates.
 * - The asset type and amount must be identical to the Asset that it points to.
 */
export class AssetOutPoint {
    readonly transactionHash: H256;
    readonly index: number;
    readonly assetType: H256;
    readonly amount: number;
    readonly lockScriptHash?: H256;
    readonly parameters?: Buffer[];

    constructor(data: AssetOutPointData) {
        const { transactionHash, index, assetType, amount, lockScriptHash, parameters } = data;
        this.transactionHash = transactionHash;
        this.index = index;
        this.assetType = assetType;
        this.amount = amount;
        this.lockScriptHash = lockScriptHash;
        this.parameters = parameters;
    }

    toEncodeObject() {
        const { transactionHash, index, assetType, amount } = this;
        return [transactionHash.toEncodeObject(), index, assetType.toEncodeObject(), amount];
    }

    static fromJSON(data: any) {
        const { transactionHash, index, assetType, amount } = data;
        return new this({
            transactionHash: new H256(transactionHash),
            index,
            assetType: new H256(assetType),
            amount,
        });
    }

    toJSON() {
        const { transactionHash, index, assetType, amount } = this;
        return {
            transactionHash: transactionHash.value,
            index,
            assetType: assetType.value,
            amount,
        };
    }
}
