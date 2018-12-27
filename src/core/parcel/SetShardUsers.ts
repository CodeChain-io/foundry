import { PlatformAddress } from "../classes";
import { Parcel } from "../Parcel";
import { NetworkId } from "../types";

export class SetShardUsers extends Parcel {
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

    protected actionToEncodeObject(): any[] {
        const { shardId, users } = this;
        return [
            6,
            shardId,
            users.map(user => user.getAccountId().toEncodeObject())
        ];
    }

    protected actionToJSON(): any {
        const { shardId, users } = this;
        return {
            shardId,
            users: users.map(user => user.value)
        };
    }

    protected action(): string {
        return "setShardUsers";
    }
}
