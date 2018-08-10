import { H160 } from "../H160";

export class ChangeShardUsers {
    public readonly shardId: number;
    public readonly users: H160[];
    constructor(params: { shardId: number, users: H160[] }) {
        const { shardId, users } = params;
        this.shardId = shardId;
        this.users = users;
    }

    toEncodeObject(): Array<any> {
        const { shardId, users } = this;
        return [6, shardId, users.map(user => user.toEncodeObject())];
    }

    toJSON() {
        const { shardId, users } = this;
        return {
            action: "changeShardUsers",
            shardId,
            users: users.map(user => user.value)
        };
    }
}
