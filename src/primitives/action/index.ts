import { NoopAction } from "./NoopAction";
import { PaymentAction } from "./PaymentAction";
import { SetRegularKeyAction } from "./SetRegularKeyAction";
import { AssetMintAction } from "./AssetMintAction";

export type Action =
    NoopAction
    | PaymentAction
    | SetRegularKeyAction
    | AssetMintAction;

export { NoopAction, PaymentAction, SetRegularKeyAction, AssetMintAction } ;
