import { AssetTransferAddress, H160 } from "codechain-primitives/lib";
import { AssetTransferOutput } from "./transaction/AssetTransferOutput";
import { U64 } from "./U64";

export type NetworkId = string;

export type AssetTransferOutputValue =
    | AssetTransferOutput
    | {
          quantity: U64 | number | string;
          assetType: H160 | string;
          shardId: number;
          recipient: AssetTransferAddress | string;
      };
