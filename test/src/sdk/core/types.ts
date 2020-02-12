import {
    AssetAddressValue,
    H160Value,
    U64,
    U64Value
} from "foundry-primitives";
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
    networkID: NetworkId;
    minPayCost: U64;
    minCreateShardCost: U64;
    minSetShardOwnersCost: U64;
    minSetShardUsersCost: U64;
    minCustomCost: U64;
    maxBodySize: U64;
    snapshotPeriod: U64;
    termSeconds: U64;
    nominationExpiration: U64;
    custodyPeriod: U64;
    releasePeriod: U64;
    maxNumOfValidators: U64;
    minNumOfValidators: U64;
    delegationThreshold: U64;
    minDeposit: U64;
    maxCandidateMetadataSize: U64;
}
