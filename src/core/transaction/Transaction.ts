import { AssetMintTransaction } from "./AssetMintTransaction";
import { AssetTransferTransaction } from "./AssetTransferTransaction";

export type Transaction =
    AssetMintTransaction
    | AssetTransferTransaction;

export const getTransactionFromJSON = (obj: { type: string, data: object }) => {
    const { type } = obj;
    switch (type) {
        case "assetMint":
            return AssetMintTransaction.fromJSON(obj);
        case "assetTransfer":
            return AssetTransferTransaction.fromJSON(obj);
        default:
            throw new Error(`Unexpected transaction type: ${type}`);
    }
};
