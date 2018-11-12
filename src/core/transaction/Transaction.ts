import {
    AssetComposeTransaction,
    AssetComposeTransactionJSON
} from "./AssetComposeTransaction";
import {
    AssetDecomposeTransaction,
    AssetDecomposeTransactionJSON
} from "./AssetDecomposeTransaction";
import {
    AssetMintTransaction,
    AssetMintTransactionJSON
} from "./AssetMintTransaction";
import {
    AssetTransferTransaction,
    AssetTransferTransactionJSON
} from "./AssetTransferTransaction";

export type TransactionJSON =
    | AssetMintTransactionJSON
    | AssetTransferTransactionJSON
    | AssetComposeTransactionJSON
    | AssetDecomposeTransactionJSON;

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
export const getTransactionFromJSON = (json: TransactionJSON): Transaction => {
    switch (json.type) {
        case "assetMint":
            return AssetMintTransaction.fromJSON(json);
        case "assetTransfer":
            return AssetTransferTransaction.fromJSON(json);
        case "assetCompose":
            return AssetComposeTransaction.fromJSON(json);
        case "assetDecompose":
            return AssetDecomposeTransaction.fromJSON(json);
        default:
            throw Error(`Unexpected transaction type: ${(json as any).type}`);
    }
};
