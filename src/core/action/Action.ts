import { PlatformAddress } from "../../key/PlatformAddress";
import { getTransactionFromJSON } from "../transaction/Transaction";
import { U256 } from "../U256";
import { H512 } from "../H512";

import { ChangeShardState } from "./ChangeShardState";
import { Payment } from "./Payment";
import { SetRegularKey } from "./SetReulgarKey";
import { CreateShard } from "./CreateShard";
import { ChangeShardOwners } from "./ChangeShardOwners";
import { ChangeShardUsers } from "./ChangeShardUsers";

export type Action = ChangeShardState | Payment | SetRegularKey | CreateShard | ChangeShardOwners | ChangeShardUsers;

export function getActionFromJSON(json: any): Action {
    const { action } = json;
    switch (action) {
        case "changeShardState":
            const { transactions } = json;
            return new ChangeShardState({ transactions: transactions.map(getTransactionFromJSON) });
        case "payment":
            const { receiver, amount } = json;
            return new Payment(PlatformAddress.ensure(receiver), new U256(amount));
        case "setRegularKey":
            const { key } = json;
            return new SetRegularKey(new H512(key));
        case "createShard":
            return new CreateShard();
        case "changeShardOwners":
            const { shardId, owners } = json;
            return new ChangeShardOwners({ shardId, owners: owners.map(PlatformAddress.ensure) });
        case "changeShardUsers": {
            const { shardId, users } = json;
            return new ChangeShardUsers({ shardId, users: users.map(PlatformAddress.ensure) });
        }
        default:
            throw new Error(`Unexpected parcel action: ${action}`);
    }
}
