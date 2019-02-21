import { PlatformAddress } from "codechain-primitives/lib";
import { Transaction } from "../Transaction";
import { NetworkId } from "../types";

/* tslint:disable:no-empty-interface */
export interface CreateShardActionJSON {}

export class CreateShard extends Transaction {
    private readonly users: PlatformAddress[];

    public constructor(
        params: { users: PlatformAddress[] },
        networkId: NetworkId
    ) {
        super(networkId);
        const { users } = params;
        this.users = users;
    }

    public type(): string {
        return "createShard";
    }

    protected actionToEncodeObject(): any[] {
        return [
            4,
            this.users.map(user => user.getAccountId().toEncodeObject())
        ];
    }

    protected actionToJSON(): CreateShardActionJSON {
        return {};
    }
}
