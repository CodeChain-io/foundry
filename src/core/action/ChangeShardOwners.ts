import { H160 } from "../H160";

export class ChangeShardOwners {
    public readonly shardId: number;
    public readonly owners: H160[];
    constructor(params: { shardId: number, owners: H160[] }) {
        const { shardId, owners } = params;
        this.shardId = shardId;
        this.owners = owners;
    }

    toEncodeObject(): Array<any> {
        const { shardId, owners } = this;
        return [5, shardId, owners.map(owner => owner.toEncodeObject())];
    }

    toJSON() {
        const { shardId, owners } = this;
        return {
            action: "changeShardOwners",
            shardId,
            owners: owners.map(owner => owner.value)
        };
    }
}
