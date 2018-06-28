import { AssetMintTransaction } from "./AssetMintTransaction";
import { AssetTransferTransaction, AssetTransferInput, AssetTransferOutput, AssetOutPoint } from "./AssetTransferTransaction";

export type Transaction =
    AssetMintTransaction
    | AssetTransferTransaction;

export { AssetMintTransaction, AssetTransferTransaction, AssetTransferInput, AssetTransferOutput, AssetOutPoint };

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
