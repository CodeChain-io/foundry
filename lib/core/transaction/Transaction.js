"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const AssetComposeTransaction_1 = require("./AssetComposeTransaction");
const AssetDecomposeTransaction_1 = require("./AssetDecomposeTransaction");
const AssetMintTransaction_1 = require("./AssetMintTransaction");
const AssetTransferTransaction_1 = require("./AssetTransferTransaction");
/**
 * Create a transaction from either an AssetMintTransaction JSON object or an
 * AssetTransferTransaction JSON object.
 * @param params Either an AssetMintTransaction JSON object or an AssetTransferTransaction JSON object.
 * @returns A Transaction.
 */
exports.getTransactionFromJSON = (params) => {
    const { type } = params;
    switch (type) {
        case "assetMint":
            return AssetMintTransaction_1.AssetMintTransaction.fromJSON(params);
        case "assetTransfer":
            return AssetTransferTransaction_1.AssetTransferTransaction.fromJSON(params);
        case "assetCompose":
            return AssetComposeTransaction_1.AssetComposeTransaction.fromJSON(params);
        case "assetDecompose":
            return AssetDecomposeTransaction_1.AssetDecomposeTransaction.fromJSON(params);
        default:
            throw Error(`Unexpected transaction type: ${type}`);
    }
};
