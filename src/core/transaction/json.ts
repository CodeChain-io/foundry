import {
    H160,
    H256,
    H512,
    PlatformAddress,
    SignedTransaction,
    Transaction,
    U64
} from "../classes";
import { AssetMintOutput } from "./AssetMintOutput";
import { AssetTransferInput } from "./AssetTransferInput";
import { AssetTransferOutput } from "./AssetTransferOutput";
import { ChangeAssetScheme } from "./ChangeAssetScheme";
import { ComposeAsset } from "./ComposeAsset";
import { CreateShard } from "./CreateShard";
import { Custom } from "./Custom";
import { DecomposeAsset } from "./DecomposeAsset";
import { IncreaseAssetSupply } from "./IncreaseAssetSupply";
import { MintAsset } from "./MintAsset";
import { OrderOnTransfer } from "./OrderOnTransfer";
import { Pay } from "./Pay";
import { Remove } from "./Remove";
import { SetRegularKey } from "./SetRegularKey";
import { SetShardOwners } from "./SetShardOwners";
import { SetShardUsers } from "./SetShardUsers";
import { Store } from "./Store";
import { TransferAsset } from "./TransferAsset";
import { UnwrapCCC } from "./UnwrapCCC";
import { WrapCCC } from "./WrapCCC";

export function fromJSONToTransaction(result: any): Transaction {
    const { seq, fee, networkId, action } = result;
    let tx;
    switch (action.type) {
        case "mintAsset": {
            const { shardId, metadata, approvals } = action;
            const approver =
                action.approver == null
                    ? null
                    : PlatformAddress.ensure(action.approver);
            const administrator =
                action.administrator == null
                    ? null
                    : PlatformAddress.ensure(action.administrator);
            const allowedScriptHashes =
                action.allowedScriptHashes == null
                    ? null
                    : action.allowedScriptHashes.map((hash: string) =>
                          H160.ensure(hash)
                      );
            const output = AssetMintOutput.fromJSON(action.output);
            tx = new MintAsset({
                networkId,
                shardId,
                metadata,
                output,
                approver,
                administrator,
                allowedScriptHashes,
                approvals
            });
            break;
        }
        case "changeAssetScheme": {
            const { metadata, approvals, shardId } = action;
            const assetType = new H160(action.assetType);
            const approver =
                action.approver == null
                    ? null
                    : PlatformAddress.ensure(action.approver);
            const administrator =
                action.administrator == null
                    ? null
                    : PlatformAddress.ensure(action.administrator);
            const allowedScriptHashes = action.allowedScriptHashes.map(
                (hash: string) => H160.ensure(hash)
            );
            tx = new ChangeAssetScheme({
                networkId,
                shardId,
                assetType,
                metadata,
                approver,
                administrator,
                allowedScriptHashes,
                approvals
            });
            break;
        }
        case "increaseAssetSupply": {
            const { approvals, shardId } = action;
            const assetType = new H160(action.assetType);
            const output = AssetMintOutput.fromJSON(action.output);
            tx = new IncreaseAssetSupply({
                networkId,
                shardId,
                assetType,
                output,
                approvals
            });
            break;
        }
        case "transferAsset": {
            const metadata = action.metadata;
            const approvals = action.approvals;
            const expiration = action.expiration;
            const burns = action.burns.map(AssetTransferInput.fromJSON);
            const inputs = action.inputs.map(AssetTransferInput.fromJSON);
            const outputs = action.outputs.map(AssetTransferOutput.fromJSON);
            const orders = action.orders.map(OrderOnTransfer.fromJSON);
            tx = new TransferAsset({
                networkId,
                burns,
                inputs,
                outputs,
                orders,
                metadata,
                approvals,
                expiration
            });
            break;
        }
        case "decomposeAsset": {
            const approvals = action.approvals;
            const input = AssetTransferInput.fromJSON(action.input);
            const outputs = action.outputs.map(AssetTransferOutput.fromJSON);
            tx = new DecomposeAsset({
                input,
                outputs,
                networkId,
                approvals
            });
            break;
        }
        case "composeAsset": {
            const { shardId, metadata, approvals } = action;
            const approver =
                action.approver == null
                    ? null
                    : PlatformAddress.ensure(action.approver);
            const administrator =
                action.administrator == null
                    ? null
                    : PlatformAddress.ensure(action.administrator);
            const allowedScriptHashes = action.allowedScriptHashes.map(
                (hash: string) => H160.ensure(hash)
            );
            const inputs = action.inputs.map(AssetTransferInput.fromJSON);
            const output = AssetMintOutput.fromJSON(action.output);
            tx = new ComposeAsset({
                networkId,
                shardId,
                metadata,
                approver,
                administrator,
                allowedScriptHashes,
                inputs,
                output,
                approvals
            });
            break;
        }
        case "unwrapCCC": {
            const burn = AssetTransferInput.fromJSON(action.burn);
            const receiver = PlatformAddress.ensure(action.receiver);
            tx = new UnwrapCCC({
                burn,
                networkId,
                receiver
            });
            break;
        }
        case "pay": {
            const receiver = PlatformAddress.ensure(action.receiver);
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
            const users = action.users.map(PlatformAddress.ensure);
            tx = new CreateShard({ users }, networkId);
            break;
        }
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
            const parameters = action.parameters.map((p: string) =>
                Buffer.from(p, "hex")
            );
            const quantity = U64.ensure(action.quantity);
            const payer = PlatformAddress.ensure(action.payer);
            tx = new WrapCCC(
                {
                    shardId,
                    lockScriptHash,
                    parameters,
                    quantity,
                    payer
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

// FIXME: any
/**
 * Create a SignedTransaction from a SignedTransaction JSON object.
 * @param data A SignedTransaction JSON object.
 * @returns A SignedTransaction.
 */
export function fromJSONToSignedTransaction(data: any) {
    const { sig, blockNumber, blockHash, transactionIndex, result } = data;
    if (typeof sig !== "string") {
        throw Error("Unexpected type of sig");
    }
    if (blockNumber) {
        return new SignedTransaction(
            fromJSONToTransaction(data),
            sig,
            blockNumber,
            new H256(blockHash),
            transactionIndex,
            result
        );
    } else {
        return new SignedTransaction(fromJSONToTransaction(data), sig);
    }
}
