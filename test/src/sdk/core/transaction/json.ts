import {
    Address,
    H160,
    H256,
    H512,
    SignedTransaction,
    Transaction,
    U64
} from "../classes";
import { SignedTransactionJSON } from "../SignedTransaction";
import { CreateShard } from "./CreateShard";
import { Custom } from "./Custom";
import { Pay } from "./Pay";
import { Remove } from "./Remove";
import { SetRegularKey } from "./SetRegularKey";
import { SetShardOwners } from "./SetShardOwners";
import { SetShardUsers } from "./SetShardUsers";
import { Store } from "./Store";

export function fromJSONToTransaction(result: any): Transaction {
    const { seq, fee, networkId, action } = result;
    let tx;
    switch (action.type) {
        case "pay": {
            const receiver = Address.ensure(action.receiver);
            const quantity = new U64(action.quantity);
            tx = new Pay(receiver, quantity, networkId);
            break;
        }
        case "setRegularKey": {
            const key = new H512(action.key);
            tx = new SetRegularKey(key, networkId);
            break;
        }
        case "createShard": {
            const users = action.users.map(Address.ensure);
            tx = new CreateShard({ users }, networkId);
            break;
        }
        case "setShardOwners": {
            const shardId = action.shardId;
            const owners = action.owners.map(Address.ensure);
            tx = new SetShardOwners(
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
            const users = action.users.map(Address.ensure);
            tx = new SetShardUsers(
                {
                    shardId,
                    users
                },
                networkId
            );
            break;
        }
        case "store": {
            const { content, signature } = action;
            const certifier = Address.ensure(action.certifier);
            tx = new Store(
                {
                    content,
                    certifier: Address.ensure(certifier),
                    signature
                },
                networkId
            );
            break;
        }
        case "remove": {
            const signature = action.signature;
            const hash = H256.ensure(action.hash);
            tx = new Remove(
                {
                    hash: H256.ensure(hash),
                    signature
                },
                networkId
            );
            break;
        }
        case "custom": {
            const handlerId = U64.ensure(action.handlerId);
            const bytes = Buffer.from(action.bytes);
            tx = new Custom(
                {
                    handlerId,
                    bytes
                },
                networkId
            );
            break;
        }
        default:
            throw Error(`Unexpected action: ${action}`);
    }
    if (seq != null) {
        tx.setSeq(seq);
    }
    if (fee != null) {
        tx.setFee(fee);
    }
    return tx;
}

/**
 * Create a SignedTransaction from a SignedTransaction JSON object.
 * @param data A SignedTransaction JSON object.
 * @returns A SignedTransaction.
 */
export function fromJSONToSignedTransaction(data: SignedTransactionJSON) {
    const {
        sig,
        signerPublic,
        blockNumber,
        blockHash,
        transactionIndex
    } = data;
    if (typeof sig !== "string") {
        throw Error("Unexpected type of sig");
    }
    if (blockNumber != null && blockHash != null && transactionIndex != null) {
        return new SignedTransaction(
            fromJSONToTransaction(data),
            sig,
            signerPublic,
            blockNumber,
            new H256(blockHash),
            transactionIndex
        );
    } else {
        return new SignedTransaction(
            fromJSONToTransaction(data),
            sig,
            signerPublic
        );
    }
}
