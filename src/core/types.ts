import { AssetAddressValue, H160Value, U64Value } from "codechain-primitives";

import { AssetTransferOutput } from "./transaction/AssetTransferOutput";

export type NetworkId = string;

export type AssetTransferOutputValue =
    | AssetTransferOutput
    | {
          quantity: U64Value;
          assetType: H160Value;
          shardId: number;
          recipient: AssetAddressValue;
      };
