import { PlatformAddress } from "../classes";
import { Transaction } from "../Transaction";
import { NetworkId } from "../types";

export interface SetShardOwnersActionJSON {
    shardId: number;
    owners: string[];
}

export class SetShardOwners extends Transaction {
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

    public type(): string {
        return "setShardOwners";
    }

    protected actionToEncodeObject(): any[] {
        const { shardId, owners } = this;
        return [
            5,
            shardId,
            owners.map(owner => owner.getAccountId().toEncodeObject())
        ];
    }

    protected actionToJSON(): SetShardOwnersActionJSON {
        const { shardId, owners } = this;
        return {
            shardId,
            owners: owners.map(owner => owner.value)
        };
    }
}
