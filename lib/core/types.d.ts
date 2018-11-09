import { AssetTransferAddress, H256 } from "codechain-primitives/lib";
import { AssetTransferOutput } from "./transaction/AssetTransferOutput";
import { U256 } from "./U256";
export declare type NetworkId = string;
export declare type AssetTransferOutputValue = AssetTransferOutput | {
    amount: U256 | number | string;
    assetType: H256 | string;
    recipient: AssetTransferAddress | string;
};
