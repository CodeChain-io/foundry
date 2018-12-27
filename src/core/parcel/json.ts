import { H160, H256, H512, Parcel, PlatformAddress, U64 } from "../classes";
import { SignedParcel } from "../classes";
import { getTransactionFromJSON } from "../transaction/Transaction";
import { AssetTransaction } from "./AssetTransaction";
import { CreateShard } from "./CreateShard";
import { Pay } from "./Pay";
import { Remove } from "./Remove";
import { SetRegularKey } from "./SetRegularKey";
import { SetShardOwners } from "./SetShardOwners";
import { SetShardUsers } from "./SetShardUsers";
import { Store } from "./Store";
import { WrapCCC } from "./WrapCCC";

export function fromJSONToParcel(result: any): Parcel {
    const { seq, fee, networkId, action } = result;
    let parcel;
    switch (action.action) {
        case "assetTransaction": {
            const transaction = getTransactionFromJSON(action.transaction);
            const { approvals } = action;
            parcel = new AssetTransaction(
                {
                    transaction,
                    approvals
                },
                networkId
            );
            break;
        }
        case "pay": {
            const receiver = PlatformAddress.ensure(action.receiver);
            const amount = new U64(action.amount);
            parcel = new Pay(receiver, amount, networkId);
            break;
        }
        case "setRegularKey": {
            const key = new H512(action.key);
            parcel = new SetRegularKey(key, networkId);
            break;
        }
        case "createShard":
            parcel = new CreateShard(networkId);
            break;
        case "setShardOwners": {
            const shardId = action.shardId;
            const owners = action.owners.map(PlatformAddress.ensure);
            parcel = new SetShardOwners(
                {
                    shardId,
                    owners
                },
                networkId
            );
            break;
        }
        case "setShardUsers": {
            const shardId = action.shardId;
            const users = action.users.map(PlatformAddress.ensure);
            parcel = new SetShardUsers(
                {
                    shardId,
                    users
                },
                networkId
            );
            break;
        }
        case "wrapCCC": {
            const shardId = action.shardId;
            const lockScriptHash = H160.ensure(action.lockScriptHash);
            const parameters = action.parameters.map((p: number[] | Buffer) =>
                Buffer.from(p)
            );
            const amount = U64.ensure(action.amount);
            parcel = new WrapCCC(
                {
                    shardId,
                    lockScriptHash,
                    parameters,
                    amount
                },
                networkId
            );
            break;
        }
        case "store": {
            const { content, signature } = action;
            const certifier = PlatformAddress.ensure(action.certifier);
            parcel = new Store(
                {
                    content,
                    certifier: PlatformAddress.ensure(certifier),
                    signature
                },
                networkId
            );
            break;
        }
        case "remove": {
            const signature = action.signature;
            const hash = H256.ensure(action.hash);
            parcel = new Remove(
                {
                    hash: H256.ensure(hash),
                    signature
                },
                networkId
            );
            break;
        }
        default:
            throw Error(`Unexpected parcel action: ${action}`);
    }
    parcel.setSeq(seq);
    parcel.setFee(fee);
    return parcel;
}

// FIXME: any
/**
 * Create a SignedParcel from a SignedParcel JSON object.
 * @param data A SignedParcel JSON object.
 * @returns A SignedParcel.
 */
export function fromJSONToSignedParcel(data: any) {
    const { sig, blockNumber, blockHash, parcelIndex } = data;
    if (typeof sig !== "string") {
        throw Error("Unexpected type of sig");
    }
    if (blockNumber) {
        return new SignedParcel(
            fromJSONToParcel(data),
            sig,
            blockNumber,
            new H256(blockHash),
            parcelIndex
        );
    } else {
        return new SignedParcel(fromJSONToParcel(data), sig);
    }
}
