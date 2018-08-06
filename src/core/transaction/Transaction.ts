import { AssetMintTransaction } from "./AssetMintTransaction";
import { AssetTransferTransaction } from "./AssetTransferTransaction";
import { CreateWorldTransaction } from "./CreateWorldTransaction";

export type Transaction =
    CreateWorldTransaction
    | AssetMintTransaction
    | AssetTransferTransaction;

/**
 * Create a transaction from either an AssetMintTransaction JSON object or an
 * AssetTransferTransaction JSON object.
 * @param params Either an AssetMintTransaction JSON object or an AssetTransferTransaction JSON object.
 * @returns A Transaction.
 */
export const getTransactionFromJSON = (params: { type: string, data: object }) => {
    const { type } = params;
    switch (type) {
        case "createWorld":
            return CreateWorldTransaction.fromJSON(params);
        case "assetMint":
            return AssetMintTransaction.fromJSON(params);
        case "assetTransfer":
            return AssetTransferTransaction.fromJSON(params);
        default:
            throw new Error(`Unexpected transaction type: ${type}`);
    }
};
