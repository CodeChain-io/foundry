import { AssetComposeTransaction } from "./AssetComposeTransaction";
import { AssetDecomposeTransaction } from "./AssetDecomposeTransaction";
import { AssetMintTransaction } from "./AssetMintTransaction";
import { AssetTransferTransaction } from "./AssetTransferTransaction";

export type Transaction =
    | AssetMintTransaction
    | AssetTransferTransaction
    | AssetComposeTransaction
    | AssetDecomposeTransaction;

/**
 * Create a transaction from either an AssetMintTransaction JSON object or an
 * AssetTransferTransaction JSON object.
 * @param params Either an AssetMintTransaction JSON object or an AssetTransferTransaction JSON object.
 * @returns A Transaction.
 */
export const getTransactionFromJSON = (params: {
    type: string;
    data: object;
}) => {
    const { type } = params;
    switch (type) {
        case "assetMint":
            return AssetMintTransaction.fromJSON(params);
        case "assetTransfer":
            return AssetTransferTransaction.fromJSON(params);
        case "assetCompose":
            return AssetComposeTransaction.fromJSON(params);
        case "assetDecompose":
            return AssetDecomposeTransaction.fromJSON(params);
        default:
            throw Error(`Unexpected transaction type: ${type}`);
    }
};
