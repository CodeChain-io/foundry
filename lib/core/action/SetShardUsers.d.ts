import { PlatformAddress } from "codechain-primitives";
export declare class SetShardUsers {
    readonly shardId: number;
    readonly users: PlatformAddress[];
    constructor(params: {
        shardId: number;
        users: PlatformAddress[];
    });
    toEncodeObject(): any[];
    toJSON(): {
        action: string;
        shardId: number;
        users: string[];
    };
}
