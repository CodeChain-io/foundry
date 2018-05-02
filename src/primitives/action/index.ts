import { NoopAction } from "./NoopAction";
import { PaymentAction } from "./PaymentAction";
import { SetRegularKeyAction } from "./SetRegularKeyAction";
import { AssetMintAction } from "./AssetMintAction";
import { AssetTransferAction } from "./AssetTransferAction";

export type Action =
    NoopAction
    | PaymentAction
    | SetRegularKeyAction
    | AssetMintAction
    | AssetTransferAction;

export { NoopAction, PaymentAction, SetRegularKeyAction, AssetMintAction, AssetTransferAction };
