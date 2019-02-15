import { PlatformAddress } from "../classes";
import { Transaction } from "../Transaction";
import { NetworkId } from "../types";

export interface SetShardUsersActionJSON {
    shardId: number;
    users: string[];
}

export class SetShardUsers extends Transaction {
    private readonly shardId: number;
    private readonly users: PlatformAddress[];
    public constructor(
        params: { shardId: number; users: PlatformAddress[] },
        networkId: NetworkId
    ) {
        super(networkId);
        this.shardId = params.shardId;
        this.users = params.users;
    }

    public type(): string {
        return "setShardUsers";
    }

    protected actionToEncodeObject(): any[] {
        const { shardId, users } = this;
        return [
            6,
            shardId,
            users.map(user => user.getAccountId().toEncodeObject())
        ];
    }

    protected actionToJSON(): SetShardUsersActionJSON {
        const { shardId, users } = this;
        return {
            shardId,
            users: users.map(user => user.value)
        };
    }
}
