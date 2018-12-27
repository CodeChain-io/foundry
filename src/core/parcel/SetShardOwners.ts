import { PlatformAddress } from "../classes";
import { Parcel } from "../Parcel";
import { NetworkId } from "../types";

export class SetShardOwners extends Parcel {
    private readonly shardId: number;
    private readonly owners: PlatformAddress[];

    public constructor(
        params: { shardId: number; owners: PlatformAddress[] },
        networkId: NetworkId
    ) {
        super(networkId);
        this.shardId = params.shardId;
        this.owners = params.owners;
    }

    protected actionToEncodeObject(): any[] {
        const { shardId, owners } = this;
        return [
            5,
            shardId,
            owners.map(owner => owner.getAccountId().toEncodeObject())
        ];
    }

    protected actionToJSON(): any {
        const { shardId, owners } = this;
        return {
            shardId,
            owners: owners.map(owner => owner.value)
        };
    }

    protected action(): string {
        return "setShardOwners";
    }
}
