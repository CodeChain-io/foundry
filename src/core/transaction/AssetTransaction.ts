import { ChangeAssetSchemeJSON } from "./ChangeAssetScheme";
import { ComposeAssetJSON } from "./ComposeAsset";
import { DecomposeAssetJSON } from "./DecomposeAsset";
import { MintAssetJSON } from "./MintAsset";
import { TransferAssetJSON } from "./TransferAsset";
import { UnwrapCCCJSON } from "./UnwrapCCC";

export type AssetTransactionJSON =
    | ChangeAssetSchemeJSON
    | ComposeAssetJSON
    | DecomposeAssetJSON
    | MintAssetJSON
    | TransferAssetJSON
    | UnwrapCCCJSON;
