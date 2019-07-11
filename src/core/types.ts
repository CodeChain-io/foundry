import {
    AssetAddressValue,
    H160Value,
    U64,
    U64Value
} from "codechain-primitives";
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

export interface CommonParams {
    maxExtraDataSize: U64;
    maxAssetSchemeMetadataSize: U64;
    maxTransferMetadataSize: U64;
    maxTextContentSize: U64;
    networkID: NetworkId;
    minPayCost: U64;
    minSetRegularKeyCost: U64;
    minCreateShardCost: U64;
    minSetShardOwnersCost: U64;
    minSetShardUsersCost: U64;
    minWrapCccCost: U64;
    minCustomCost: U64;
    minStoreCost: U64;
    minRemoveCost: U64;
    minMintAssetCost: U64;
    minTransferAssetCost: U64;
    minChangeAssetSchemeCost: U64;
    minIncreaseAssetSupplyCost: U64;
    minComposeAssetCost: U64;
    minDecomposeAssetCost: U64;
    minUnwrapCccCost: U64;
    maxBodySize: U64;
    snapshotPeriod: U64;
}
