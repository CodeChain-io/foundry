import { PlatformAddress } from "codechain-primitives";

import { H512 } from "../H512";
import { getTransactionFromJSON } from "../transaction/Transaction";
import { U256 } from "../U256";

import { AssetTransactionGroup } from "./AssetTransactionGroup";
import { CreateShard } from "./CreateShard";
import { Payment } from "./Payment";
import { SetRegularKey } from "./SetReulgarKey";
import { SetShardOwners } from "./SetShardOwners";
import { SetShardUsers } from "./SetShardUsers";

export type Action =
    | AssetTransactionGroup
    | Payment
    | SetRegularKey
    | CreateShard
    | SetShardOwners
    | SetShardUsers;

export function getActionFromJSON(json: any): Action {
    const { action } = json;
    switch (action) {
        case "assetTransactionGroup":
            const { transactions } = json;
            return new AssetTransactionGroup({
                transactions: transactions.map(getTransactionFromJSON)
            });
        case "payment":
            const { receiver, amount } = json;
            return new Payment(
                PlatformAddress.ensure(receiver),
                new U256(amount)
            );
        case "setRegularKey":
            const { key } = json;
            return new SetRegularKey(new H512(key));
        case "createShard":
            return new CreateShard();
        case "setShardOwners": {
            const { shardId, owners } = json;
            return new SetShardOwners({
                shardId,
                owners: owners.map(PlatformAddress.ensure)
            });
        }
        case "setShardUsers": {
            const { shardId, users } = json;
            return new SetShardUsers({
                shardId,
                users: users.map(PlatformAddress.ensure)
            });
        }
        default:
            throw Error(`Unexpected parcel action: ${action}`);
    }
}
