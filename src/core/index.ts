// FIXME: Use interface instead of importing key class.
import { AssetTransferAddress, PlatformAddress } from "../key/classes";

import { Parcel } from "./Parcel";
import { Asset } from "./Asset";
import { U256 } from "./U256";
import { H160 } from "./H160";
import { H512 } from "./H512";
import { Transaction, getTransactionFromJSON } from "./transaction/Transaction";
import { AssetScheme } from "./AssetScheme";
import { H256 } from "./H256";
import { Invoice } from "./Invoice";
import { Block } from "./Block";
import { SignedParcel } from "./SignedParcel";
import { Payment } from "./action/Payment";
import { SetRegularKey } from "./action/SetReulgarKey";
import { ChangeShardState } from "./action/ChangeShardState";
import { CreateShard } from "./action/CreateShard";
import { SetShardOwners } from "./action/SetShardOwners";
import { SetShardUsers } from "./action/SetShardUsers";
import { AssetTransferInput } from "./transaction/AssetTransferInput";
import { AssetTransferOutput } from "./transaction/AssetTransferOutput";
import { AssetOutPoint } from "./transaction/AssetOutPoint";
import { AssetMintTransaction } from "./transaction/AssetMintTransaction";
import { AssetTransferTransaction } from "./transaction/AssetTransferTransaction";
import { CreateWorldTransaction } from "./transaction/CreateWorldTransaction";
import { SetWorldOwnersTransaction } from "./transaction/SetWorldOwnersTransaction";
import { SetWorldUsersTransaction } from "./transaction/SetWorldUsersTransaction";
import { Script } from "./Script";

type NetworkId = string;

export class Core {
    private networkId: NetworkId;

    /**
     * @param params.networkId The network id of CodeChain.
     */
    constructor(params: { networkId: NetworkId }) {
        const { networkId } = params;
        this.networkId = networkId;
    }

    /**
     * Creates Payment action which pays the value amount of CCC(CodeChain Coin)
     * from the parcel signer to the recipient. Who is signing the parcel will pay.
     * @param params.recipient The platform account who receives CCC
     * @param params.amount Amount of CCC to pay
     * @throws Given string for recipient is invalid for converting it to PlatformAddress
     * @throws Given number or string for amount is invalid for converting it to U256
     */
    createPaymentParcel(params: { recipient: PlatformAddress | string, amount: U256 | number | string }): Parcel {
        const { recipient, amount } = params;
        const action = new Payment(
            PlatformAddress.ensure(recipient),
            U256.ensure(amount)
        );
        return new Parcel(
            this.networkId,
            action
        );
    }

    /**
     * Creates SetRegularKey action which sets the regular key of the parcel signer.
     * @param params.key The public key of a regular key
     * @throws Given string for key is invalid for converting it to H512
     */
    createSetRegularKeyParcel(params: { key: H512 | string }): Parcel {
        const { key } = params;
        const action = new SetRegularKey(H512.ensure(key));
        return new Parcel(
            this.networkId,
            action
        );
    }

    /**
     * Creates ChangeShardState action which can mint or transfer assets through
     * AssetMintTransaction or AssetTransferTransaction.
     * @param params.transactions List of transaction
     */
    createChangeShardStateParcel(params: { transactions: Transaction[] }): Parcel {
        const { transactions } = params;
        const action = new ChangeShardState({ transactions });
        return new Parcel(
            this.networkId,
            action
        );
    }

    /**
     * Creates CreateShard action which can create new shard
     */
    createCreateShardParcel(): Parcel {
        const action = new CreateShard();
        return new Parcel(
            this.networkId,
            action
        );
    }

    createSetShardOwnersParcel(params: { shardId: number, owners: (PlatformAddress | string)[] }): Parcel {
        const { shardId, owners } = params;
        const action = new SetShardOwners({ shardId, owners: owners.map(PlatformAddress.ensure) });
        return new Parcel(
            this.networkId,
            action
        );
    }

    /**
     * Create SetShardUser action which can change shard users
     * @param params.shardId
     * @param params.users
     */
    createSetShardUsersParcel(params: { shardId: number, users: (PlatformAddress | string)[] }): Parcel {
        const { shardId, users } = params;
        const action = new SetShardUsers({ shardId, users: users.map(PlatformAddress.ensure) });
        return new Parcel(
            this.networkId,
            action
        );
    }

    /**
     * Creates asset's scheme.
     * @param params.metadata Any string that describing the asset. For example,
     * stringified JSON containing properties.
     * @param params.amount Total amount of this asset
     * @param params.registrar Platform account or null. If account is present, the
     * parcel that includes AssetTransferTransaction of this asset must be signed by
     * the registrar account.
     * @throws Given string for registrar is invalid for converting it to paltform account
     */
    createAssetScheme(params: {
        shardId: number,
        worldId: number,
        metadata: string,
        amount: number,
        registrar: PlatformAddress | string | null
    }): AssetScheme {
        const { shardId, worldId, metadata, amount, registrar } = params;
        return new AssetScheme({
            networkId: this.networkId,
            shardId,
            worldId,
            metadata,
            amount,
            registrar: registrar == null ? null : PlatformAddress.ensure(registrar),
        });
    }

    createCreateWorldTransaction(params: {
        networkId?: NetworkId;
        shardId: number,
        owners: (PlatformAddress | string)[],
        nonce?: number,
    }): CreateWorldTransaction {
        const { networkId, shardId, owners, nonce } = params;

        return new CreateWorldTransaction({
            networkId: networkId || this.networkId,
            shardId,
            owners: owners.map(PlatformAddress.ensure),
            nonce: nonce || 0
        });
    }

    createSetWorldOwnersTransaction(params: {
        networkId?: NetworkId;
        shardId: number,
        worldId: number,
        owners: (PlatformAddress | string)[],
        nonce: number,
    }): SetWorldOwnersTransaction {
        const { networkId, shardId, worldId, owners, nonce } = params;

        return new SetWorldOwnersTransaction({
            networkId: networkId || this.networkId,
            shardId,
            worldId,
            owners: owners.map(PlatformAddress.ensure),
            nonce
        });
    }

    createSetWorldUsersTransaction(params: {
        networkId?: NetworkId;
        shardId: number,
        worldId: number,
        users: (PlatformAddress | string)[],
        nonce: number,
    }): SetWorldUsersTransaction {
        const { networkId, shardId, worldId, users, nonce } = params;

        return new SetWorldUsersTransaction({
            networkId: networkId || this.networkId,
            shardId,
            worldId,
            users: users.map(PlatformAddress.ensure),
            nonce
        });
    }

    createAssetMintTransaction(params: {
        scheme: AssetScheme | {
            networkId?: NetworkId;
            shardId: number,
            worldId: number,
            metadata: string,
            registrar?: PlatformAddress | string,
            amount: number | null,
        },
        recipient: AssetTransferAddress | string,
        nonce?: number,
    }): AssetMintTransaction {
        const { scheme, recipient, nonce } = params;
        const { networkId, shardId, worldId, metadata, registrar, amount } = scheme;
        return new AssetMintTransaction({
            networkId: networkId || this.networkId,
            shardId,
            worldId,
            nonce: nonce || 0,
            registrar: registrar == null ? null : PlatformAddress.ensure(registrar),
            metadata,
            output: {
                amount,
                ...AssetTransferAddress.ensure(recipient).getLockScriptHashAndParameters()
            },
        });
    }

    createAssetTransferTransaction(params: {
        burns: AssetTransferInput[],
        inputs: AssetTransferInput[],
        outputs: AssetTransferOutput[],
        networkId?: NetworkId,
        nonce?: number,
    } = { burns: [], inputs: [], outputs: [] }): AssetTransferTransaction {
        const { burns, inputs, outputs, networkId, nonce } = params;
        return new AssetTransferTransaction({
            burns,
            inputs,
            outputs,
            networkId: networkId || this.networkId,
            nonce: nonce || 0,
        });
    }

    createAssetTransferInput(params: {
        assetOutPoint: AssetOutPoint | {
            transactionHash: H256 | string,
            index: number,
            assetType: H256 | string,
            amount: number,
            lockScriptHash?: H256 | string,
            parameters?: Buffer[],
        },
        lockScript?: Buffer,
        unlockScript?: Buffer
    }): AssetTransferInput {
        const { assetOutPoint, lockScript, unlockScript } = params;
        return new AssetTransferInput({
            prevOut: assetOutPoint instanceof AssetOutPoint
                ? assetOutPoint
                : new AssetOutPoint({
                    transactionHash: H256.ensure(assetOutPoint.transactionHash),
                    index: assetOutPoint.index,
                    assetType: H256.ensure(assetOutPoint.assetType),
                    amount: assetOutPoint.amount,
                    lockScriptHash: assetOutPoint.lockScriptHash ? H256.ensure(assetOutPoint.lockScriptHash) : undefined,
                    parameters: assetOutPoint.parameters
                }),
            lockScript,
            unlockScript
        });
    }

    createAssetOutPoint(params: {
        transactionHash: H256 | string,
        index: number,
        assetType: H256 | string,
        amount: number,
    }): AssetOutPoint {
        const { transactionHash, index, assetType, amount } = params;
        return new AssetOutPoint({
            transactionHash: H256.ensure(transactionHash),
            index,
            assetType: H256.ensure(assetType),
            amount
        });
    }

    createAssetTransferOutput(params: {
        recipient: AssetTransferAddress | string
        assetType: H256 | string,
        amount: number,
    }): AssetTransferOutput {
        const { recipient, assetType, amount } = params;
        return new AssetTransferOutput({
            ...AssetTransferAddress.ensure(recipient).getLockScriptHashAndParameters(),
            assetType: H256.ensure(assetType),
            amount,
        });
    }

    // FIXME: any
    getTransactionFromJSON(json: any): Transaction {
        return getTransactionFromJSON(json);
    }

    public classes = Core.classes;
    static classes = {
        // Data
        H160,
        H256,
        H512,
        U256,
        Invoice,
        // Block
        Block,
        // Parcel
        Parcel,
        SignedParcel,
        // Action
        Payment,
        SetRegularKey,
        ChangeShardState,
        CreateShard,
        SetShardOwners,
        SetShardUsers,
        // Transaction
        AssetMintTransaction,
        AssetTransferTransaction,
        AssetTransferInput,
        AssetTransferOutput,
        AssetOutPoint,
        CreateWorldTransaction,
        SetWorldOwnersTransaction,
        // Asset and AssetScheme
        Asset,
        AssetScheme,
        // Script
        Script,
    };
}
