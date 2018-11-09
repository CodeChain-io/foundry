"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const H256_1 = require("../H256");
const U256_1 = require("../U256");
/**
 * AssetOutPoint consists of transactionHash and index, asset type, and amount.
 *
 * - The transaction that it points to must be either AssetMint or AssetTransfer.
 * - Index is what decides which Asset to point to amongst the Asset list that transaction creates.
 * - The asset type and amount must be identical to the Asset that it points to.
 */
class AssetOutPoint {
    /**
     * Create an AssetOutPoint from an AssetOutPoint JSON object.
     * @param data An AssetOutPoint JSON object.
     * @returns An AssetOutPoint.
     */
    static fromJSON(data) {
        const { transactionHash, index, assetType, amount } = data;
        return new this({
            transactionHash: new H256_1.H256(transactionHash),
            index,
            assetType: new H256_1.H256(assetType),
            amount: U256_1.U256.ensure(amount)
        });
    }
    /**
     * @param data.transactionHash A transaction hash where the Asset is created.
     * @param data.index The index in the output of the transaction.
     * @param data.assetType The asset type of the asset that it points to.
     * @param data.amount The asset amount of the asset that it points to.
     * @param data.lockScriptHash The lock script hash of the asset.
     * @param data.parameters The parameters of the asset.
     */
    constructor(data) {
        const { transactionHash, index, assetType, amount, lockScriptHash, parameters } = data;
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
    toEncodeObject() {
        const { transactionHash, index, assetType, amount } = this;
        return [
            transactionHash.toEncodeObject(),
            index,
            assetType.toEncodeObject(),
            amount.toEncodeObject()
        ];
    }
    /**
     * Convert to an AssetOutPoint JSON object.
     * @returns An AssetOutPoint JSON object.
     */
    toJSON() {
        const { transactionHash, index, assetType, amount } = this;
        return {
            transactionHash: transactionHash.value,
            index,
            assetType: assetType.value,
            amount: amount.toEncodeObject()
        };
    }
}
exports.AssetOutPoint = AssetOutPoint;
