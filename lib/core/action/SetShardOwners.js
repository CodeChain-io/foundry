"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
class SetShardOwners {
    constructor(params) {
        const { shardId, owners } = params;
        this.shardId = shardId;
        this.owners = owners;
    }
    toEncodeObject() {
        const { shardId, owners } = this;
        return [
            5,
            shardId,
            owners.map(owner => owner.getAccountId().toEncodeObject())
        ];
    }
    toJSON() {
        const { shardId, owners } = this;
        return {
            action: "setShardOwners",
            shardId,
            owners: owners.map(owner => owner.value)
        };
    }
}
exports.SetShardOwners = SetShardOwners;
