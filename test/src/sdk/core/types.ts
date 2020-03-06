import { H160Value, U64, U64Value } from "foundry-primitives";

export type NetworkId = string;

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
