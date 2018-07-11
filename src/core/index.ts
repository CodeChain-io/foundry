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

/**
 * @hidden
 */
export type ParcelParams = {
    nonce: U256 | number | string;
    fee: U256 | number | string;
};

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
     * @param params.nonce Nonce for the parcel
     * @param params.fee Fee for the parcel
     * @throws Given string for recipient is invalid for converting it to H160
     * @throws Given number or string for amount is invalid for converting it to U256
     * @throws Given number or string for nonce is invalid for converting it to U256
     * @throws Given number or string for fee is invalid for converting it to U256
     */
    createPaymentParcel(params: { recipient: H160 | string, amount: U256 | number | string } & ParcelParams): Parcel {
        const { recipient, amount, fee, nonce } = params;
        const action = new Payment(
            H160.ensure(recipient),
            U256.ensure(amount)
        );
        return new Parcel(
            U256.ensure(nonce),
            U256.ensure(fee),
            this.networkId,
            action
        );
    }

    /**
     * Creates SetRegularKey action which sets the regular key of the parcel signer.
     * @param params.key The public key of a regular key
     * @param params.nonce Nonce for the parcel
     * @param params.fee Fee for the parcel
     * @throws Given string for key is invalid for converting it to H512
     * @throws Given number or string for nonce is invalid for converting it to U256
     * @throws Given number or string for fee is invalid for converting it to U256
     */
    createSetRegularKeyParcel(params: { key: H512 | string } & ParcelParams): Parcel {
        const { key, nonce, fee } = params;
        const action = new SetRegularKey(H512.ensure(key));
        return new Parcel(
            U256.ensure(nonce),
            U256.ensure(fee),
            this.networkId,
            action
        );
    }

    /**
     * Creates ChangeShardState action which can mint or transfer assets through
     * AssetMintTransaction or AssetTransferTransaction.
     * @param params.transactions List of transaction
     * @param params.nonce Nonce for the parcel
     * @param params.fee Fee for the parcel
     * @throws Given number or string for nonce is invalid for converting it to U256
     * @throws Given number or string for fee is invalid for converting it to U256
     */
    createChangeShardStateParcel(params: { transactions: Transaction[] } & ParcelParams): Parcel {
        const { transactions, nonce, fee } = params;
        const action = new ChangeShardState(transactions);
        return new Parcel(
            U256.ensure(nonce),
            U256.ensure(fee),
            this.networkId,
            action
        );
    }

    /**
     * Creates CreateShard action which can create new shard
     * @param params.nonce Nonce for the parcel
     * @param params.fee Fee for the parcel
     * @throws Given number or string for nonce is invalid for converting it to U256
     * @throws Given number or string for fee is invalid for converting it to U256
     */
    createCreateShardParcel(params: {} & ParcelParams): Parcel {
        const { nonce, fee } = params;
        const action = new CreateShard();
        return new Parcel(
            U256.ensure(nonce),
            U256.ensure(fee),
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
    createAssetScheme(params: { metadata: string, amount: number, registrar: H160 | string | null }): AssetScheme {
        const { metadata, amount, registrar } = params;
        return new AssetScheme({
            networkId: this.networkId,
            metadata,
            amount,
            registrar: registrar === null ? null : H160.ensure(registrar)
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
