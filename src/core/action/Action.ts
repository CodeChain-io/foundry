import { PlatformAddress } from "codechain-primitives";

import { H160 } from "../H160";
import { H256 } from "../H256";
import { H512 } from "../H512";
import { getTransactionFromJSON } from "../transaction/Transaction";
import { U64 } from "../U64";

import { AssetTransaction } from "./AssetTransaction";
import { CreateShard } from "./CreateShard";
import { Payment } from "./Payment";
import { Remove } from "./Remove";
import { SetRegularKey } from "./SetRegularKey";
import { SetShardOwners } from "./SetShardOwners";
import { SetShardUsers } from "./SetShardUsers";
import { Store } from "./Store";
import { WrapCCC } from "./WrapCCC";

export type Action =
    | AssetTransaction
    | Payment
    | SetRegularKey
    | CreateShard
    | SetShardOwners
    | SetShardUsers
    | WrapCCC
    | Store
    | Remove;

export function getActionFromJSON(json: any): Action {
    const { action } = json;
    switch (action) {
        case "assetTransaction": {
            const { transaction, approvals } = json;
            return new AssetTransaction({
                transaction: getTransactionFromJSON(transaction),
                approvals
            });
        }
        case "payment": {
            const { receiver, amount } = json;
            return new Payment(
                PlatformAddress.ensure(receiver),
                new U64(amount)
            );
        }
        case "setRegularKey": {
            const { key } = json;
            return new SetRegularKey(new H512(key));
        }
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
        case "wrapCCC": {
            const { shardId, lockScriptHash, parameters, amount } = json;
            return new WrapCCC({
                shardId,
                lockScriptHash: H160.ensure(lockScriptHash),
                parameters: parameters.map((p: number[] | Buffer) =>
                    Buffer.from(p)
                ),
                amount: U64.ensure(amount)
            });
        }
        case "store": {
            const { content, certifier, signature } = json;
            return new Store({
                content,
                certifier: PlatformAddress.ensure(certifier),
                signature
            });
        }
        case "remove": {
            const { hash, signature } = json;
            return new Remove({
                hash: H256.ensure(hash),
                signature
            });
        }
        default:
            throw Error(`Unexpected parcel action: ${action}`);
    }
}
