import { NoopTransaction } from "./NoopTransaction";
import { PaymentTransaction } from "./PaymentTransaction";
import { SetRegularKeyTransaction } from "./SetRegularKeyTransaction";
import { AssetMintTransaction } from "./AssetMintTransaction";
import { AssetTransferTransaction } from "./AssetTransferTransaction";

export type Transaction =
    NoopTransaction
    | PaymentTransaction
    | SetRegularKeyTransaction
    | AssetMintTransaction
    | AssetTransferTransaction;

export { NoopTransaction, PaymentTransaction, SetRegularKeyTransaction, AssetMintTransaction, AssetTransferTransaction };
