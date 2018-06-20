import { AssetMintTransaction } from "./AssetMintTransaction";
import { AssetTransferTransaction, AssetTransferInput, AssetTransferOutput, AssetOutPoint } from "./AssetTransferTransaction";

export type Transaction =
    AssetMintTransaction
    | AssetTransferTransaction;

export { AssetMintTransaction, AssetTransferTransaction, AssetTransferInput, AssetTransferOutput, AssetOutPoint };

export const getTransactionFromJSON = (obj: string | any) => {
    const keys = Object.keys(obj);
    if (keys.length !== 1) {
        throw new Error(`Unexpected transaction keys: ${keys}`);
    }
    const type = keys[0];
    switch (type) {
    case "assetMint":
        return AssetMintTransaction.fromJSON(obj);
    case "assetTransfer":
        return AssetTransferTransaction.fromJSON(obj);
    default:
        throw new Error(`Unexpected transaction type: ${type}`);
    }
};
