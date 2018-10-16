import { AssetTransferAddress, H256 } from "codechain-primitives/lib";
import { AssetTransferOutput } from "./transaction/AssetTransferOutput";

export type NetworkId = string;

export type AssetTransferOutputValue =
    | AssetTransferOutput
    | {
          amount: number;
          assetType: H256 | string;
          recipient: AssetTransferAddress | string;
      };
