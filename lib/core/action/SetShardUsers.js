"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
class SetShardUsers {
    constructor(params) {
        const { shardId, users } = params;
        this.shardId = shardId;
        this.users = users;
    }
    toEncodeObject() {
        const { shardId, users } = this;
        return [
            6,
            shardId,
            users.map(user => user.getAccountId().toEncodeObject())
        ];
    }
    toJSON() {
        const { shardId, users } = this;
        return {
            action: "setShardUsers",
            shardId,
            users: users.map(user => user.value)
        };
    }
}
exports.SetShardUsers = SetShardUsers;
