import { PlatformAddress } from "codechain-primitives";
export declare class SetShardOwners {
    readonly shardId: number;
    readonly owners: PlatformAddress[];
    constructor(params: {
        shardId: number;
        owners: PlatformAddress[];
    });
    toEncodeObject(): any[];
    toJSON(): {
        action: string;
        shardId: number;
        owners: string[];
    };
}
