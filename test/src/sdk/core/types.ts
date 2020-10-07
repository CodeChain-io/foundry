import { U64 } from "../../primitives/src";

export type NetworkId = string;

export interface CommonParams {
    maxExtraDataSize: U64;
    networkID: NetworkId;
    maxBodySize: U64;
    snapshotPeriod: U64;
}
