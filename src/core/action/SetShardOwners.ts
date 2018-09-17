import { PlatformAddress } from "codechain-primitives";

export class SetShardOwners {
    public readonly shardId: number;
    public readonly owners: PlatformAddress[];
    constructor(params: { shardId: number; owners: PlatformAddress[] }) {
        const { shardId, owners } = params;
        this.shardId = shardId;
        this.owners = owners;
    }

    public toEncodeObject(): any[] {
        const { shardId, owners } = this;
        return [
            5,
            shardId,
            owners.map(owner => owner.getAccountId().toEncodeObject())
        ];
    }

    public toJSON() {
        const { shardId, owners } = this;
        return {
            action: "setShardOwners",
            shardId,
            owners: owners.map(owner => owner.value)
        };
    }
}
