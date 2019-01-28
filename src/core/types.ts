import { AssetTransferAddress, H160, U64 } from "codechain-primitives";

import { AssetTransferOutput } from "./transaction/AssetTransferOutput";

export type NetworkId = string;

export type AssetTransferOutputValue =
    | AssetTransferOutput
    | {
          quantity: U64 | number | string;
          assetType: H160 | string;
          shardId: number;
          recipient: AssetTransferAddress | string;
      };
