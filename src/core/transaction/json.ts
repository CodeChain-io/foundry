import {
    H160,
    H256,
    H512,
    PlatformAddress,
    SignedTransaction,
    Transaction,
    U64
} from "../classes";
import { AssetTransactionJSON } from "./AssetTransaction";
import { ChangeAssetScheme } from "./ChangeAssetScheme";
import { ComposeAsset } from "./ComposeAsset";
import { CreateShard } from "./CreateShard";
import { DecomposeAsset } from "./DecomposeAsset";
import { MintAsset } from "./MintAsset";
import { Pay } from "./Pay";
import { Remove } from "./Remove";
import { SetRegularKey } from "./SetRegularKey";
import { SetShardOwners } from "./SetShardOwners";
import { SetShardUsers } from "./SetShardUsers";
import { Store } from "./Store";
import { TransferAsset } from "./TransferAsset";
import { UnwrapCCC } from "./UnwrapCCC";
import { WrapCCC } from "./WrapCCC";

/**
 * Create a transaction from either an AssetMintTransaction JSON object or an
 * AssetTransferTransaction JSON object.
 * @param json Either an AssetMintTransaction JSON object or an AssetTransferTransaction JSON object.
 * @returns A Transaction.
 */
export const fromJSONToAssetTransaction = (
    json: AssetTransactionJSON,
    approvals: string[] = []
): any => {
    switch (json.type) {
        case "assetMint":
            return MintAsset.fromJSON(json, approvals);
        case "assetTransfer":
            return TransferAsset.fromJSON(json, approvals);
        case "assetCompose":
            return ComposeAsset.fromJSON(json, approvals);
        case "assetDecompose":
            return DecomposeAsset.fromJSON(json, approvals);
        case "assetUnwrapCCC":
            return UnwrapCCC.fromJSON(json, approvals);
        case "assetSchemeChange":
            return ChangeAssetScheme.fromJSON(json, approvals);
        default:
            throw Error(`Unexpected transaction type: ${(json as any).type}`);
    }
};

export function fromJSONToTransaction(result: any): Transaction {
    const { seq, fee, networkId, action } = result;
    let tx;
    switch (action.action) {
        case "assetTransaction": {
            const { approvals, transaction } = action;
            tx = fromJSONToAssetTransaction(transaction, approvals);
            break;
        }
        case "pay": {
            const receiver = PlatformAddress.ensure(action.receiver);
            const amount = new U64(action.amount);
            tx = new Pay(receiver, amount, networkId);
            break;
        }
        case "setRegularKey": {
            const key = new H512(action.key);
            tx = new SetRegularKey(key, networkId);
            break;
        }
        case "createShard":
            tx = new CreateShard(networkId);
            break;
        case "setShardOwners": {
            const shardId = action.shardId;
            const owners = action.owners.map(PlatformAddress.ensure);
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
            const users = action.users.map(PlatformAddress.ensure);
            tx = new SetShardUsers(
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
            tx = new WrapCCC(
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
            tx = new Store(
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
            tx = new Remove(
                {
                    hash: H256.ensure(hash),
                    signature
                },
                networkId
            );
            break;
        }
        default:
            throw Error(`Unexpected action: ${action}`);
    }
    tx.setSeq(seq);
    tx.setFee(fee);
    return tx;
}

// FIXME: any
/**
 * Create a SignedTransaction from a SignedTransaction JSON object.
 * @param data A SignedTransaction JSON object.
 * @returns A SignedTransaction.
 */
export function fromJSONToSignedTransaction(data: any) {
    const { sig, blockNumber, blockHash, parcelIndex } = data;
    if (typeof sig !== "string") {
        throw Error("Unexpected type of sig");
    }
    if (blockNumber) {
        return new SignedTransaction(
            fromJSONToTransaction(data),
            sig,
            blockNumber,
            new H256(blockHash),
            parcelIndex
        );
    } else {
        return new SignedTransaction(fromJSONToTransaction(data), sig);
    }
}
