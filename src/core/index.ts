// FIXME: Use interface instead of importing key class.
import { AssetTransferAddress } from "../key/classes";

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
import { AssetTransferInput } from "./transaction/AssetTransferInput";
import { AssetTransferOutput } from "./transaction/AssetTransferOutput";
import { AssetOutPoint } from "./transaction/AssetOutPoint";
import { AssetMintTransaction } from "./transaction/AssetMintTransaction";
import { AssetTransferTransaction } from "./transaction/AssetTransferTransaction";
import { Script } from "./Script";

export class Core {
    private networkId: number;

    /**
     * @param params.networkId The network id of CodeChain.
     */
    constructor(params: { networkId: number }) {
        const { networkId } = params;
        this.networkId = networkId;
    }

    /**
     * Creates Payment action which pays the value amount of CCC(CodeChain Coin)
     * from the parcel signer to the recipient. Who is signing the parcel will pay.
     * @param params.recipient The platform account who receives CCC
     * @param params.amount Amount of CCC to pay
     * @throws Given string for recipient is invalid for converting it to H160
     * @throws Given number or string for amount is invalid for converting it to U256
     */
    createPaymentParcel(params: { recipient: H160 | string, amount: U256 | number | string }): Parcel {
        const { recipient, amount } = params;
        const action = new Payment(
            H160.ensure(recipient),
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

    /**
     * Creates asset's scheme.
     * @param params.metadata Any string that describing the asset. For example,
     * stringified JSON containing properties.
     * @param params.amount Total amount of this asset
     * @param params.registrar Platform account or null. If account is present, the
     * parcel that includes AssetTransferTransaction of this asset must be signed by
     * the registrar account.
     * @throws Given string for registrar is invalid for converting it to H160
     */
    createAssetScheme(params: { shardId: number, metadata: string, amount: number, registrar: H160 | string | null }): AssetScheme {
        const { shardId, metadata, amount, registrar } = params;
        return new AssetScheme({
            networkId: this.networkId,
            shardId,
            metadata,
            amount,
            registrar: registrar === null ? null : H160.ensure(registrar)
        });
    }

    createAssetMintTransaction(params: {
        shardId: number,
        metadata: string,
        recipient: AssetTransferAddress | string,
        amount: number | null,
        registrar?: H160,
        nonce?: number,
        networkId?: number;
    }): AssetMintTransaction {
        const { shardId, metadata, recipient, registrar, amount, nonce, networkId } = params;
        return new AssetMintTransaction({
            networkId: networkId || this.networkId,
            shardId,
            nonce: nonce || 0,
            registrar: registrar || null,
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
        networkId?: number,
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
        // Transaction
        AssetMintTransaction,
        AssetTransferTransaction,
        AssetTransferInput,
        AssetTransferOutput,
        AssetOutPoint,
        // Asset and AssetScheme
        Asset,
        AssetScheme,
        // Script
        Script,
    };
}
