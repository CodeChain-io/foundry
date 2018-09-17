import { PlatformAddress } from "codechain-primitives";

export class SetShardUsers {
    public readonly shardId: number;
    public readonly users: PlatformAddress[];
    constructor(params: { shardId: number; users: PlatformAddress[] }) {
        const { shardId, users } = params;
        this.shardId = shardId;
        this.users = users;
    }

    public toEncodeObject(): any[] {
        const { shardId, users } = this;
        return [
            6,
            shardId,
            users.map(user => user.getAccountId().toEncodeObject())
        ];
    }

    public toJSON() {
        const { shardId, users } = this;
        return {
            action: "setShardUsers",
            shardId,
            users: users.map(user => user.value)
        };
    }
}
