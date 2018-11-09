import { AssetComposeTransaction } from "./AssetComposeTransaction";
import { AssetDecomposeTransaction } from "./AssetDecomposeTransaction";
import { AssetMintTransaction } from "./AssetMintTransaction";
import { AssetTransferTransaction } from "./AssetTransferTransaction";
export declare type Transaction = AssetMintTransaction | AssetTransferTransaction | AssetComposeTransaction | AssetDecomposeTransaction;
/**
 * Create a transaction from either an AssetMintTransaction JSON object or an
 * AssetTransferTransaction JSON object.
 * @param params Either an AssetMintTransaction JSON object or an AssetTransferTransaction JSON object.
 * @returns A Transaction.
 */
export declare const getTransactionFromJSON: (params: {
    type: string;
    data: object;
}) => AssetTransferTransaction | AssetMintTransaction | AssetComposeTransaction | AssetDecomposeTransaction;
