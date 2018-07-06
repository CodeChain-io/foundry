import {
    H160, U256, Parcel, H512, SignedParcel, AssetScheme, Block, H256,
    Invoice
} from "../primitives";
import { CreateShard, Payment, SetRegularKey, ChangeShardState } from
    "../primitives/Parcel";
import { AssetAgent, Asset } from "../primitives/Asset";
import { PubkeyAssetAgent } from "../signer/PubkeyAssetAgent";
import { MemoryKeyStore } from "../signer/MemoryKeyStore";
import {
    Transaction, getTransactionFromJSON, AssetMintTransaction,
    AssetTransferTransaction, AssetTransferOutput, AssetTransferInput,
    AssetOutPoint
} from "../primitives/transaction";
import { PlatformAddress } from "../PlatformAddress";
import { AssetTransferAddress } from "../AssetTransferAddress";

/**
 * @hidden
 */
export type ParcelParams = {
    nonce: U256 | number | string;
    fee: U256 | number | string;
};

export class Core {
    private networkId: number;
    private assetAgent: AssetAgent;

    /**
     * @param params.networkId The network id of CodeChain.
     */
    constructor(params: { networkId: number }) {
        const { networkId } = params;
        this.assetAgent = new PubkeyAssetAgent({ keyStore: new MemoryKeyStore() });
        this.networkId = networkId;
    }

    /**
     * Creates Payment action which pays the value amount of CCC(CodeChain Coin)
     * from the parcel signer to the recipient. Who is signing the parcel will pay.
     * @param params.recipient The platform account who receives CCC
     * @param params.value Amount of CCC to pay
     * @param params.nonce Nonce for the parcel
     * @param params.fee Fee for the parcel
     * @throws Given string for recipient is invalid for converting it to H160
     * @throws Given number or string for value is invalid for converting it to U256
     * @throws Given number or string for nonce is invalid for converting it to U256
     * @throws Given number or string for fee is invalid for converting it to U256
     */
    createPaymentParcel(params: { recipient: H160 | string, value: U256 | number | string } & ParcelParams): Parcel {
        const { recipient, value, fee, nonce } = params;
        const action = new Payment(
            H160.ensure(recipient),
            U256.ensure(value)
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
            metadata,
            amount,
            registrar: registrar === null ? null : H160.ensure(registrar)
        });
    }

    /**
     * Gets AssetAgent. AssetAgent manages addresses, scripts and keys for
     * locking/unlocking assets.
     */
    getAssetAgent(): AssetAgent {
        return this.assetAgent;
    }

    // FIXME: any
    getTransactionFromJSON(json: any): Transaction {
        return getTransactionFromJSON(json);
    }

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
        // Platform Address
        PlatformAddress,
        // Transaction
        AssetMintTransaction,
        AssetTransferTransaction,
        AssetTransferInput,
        AssetTransferOutput,
        AssetOutPoint,
        // Asset and AssetScheme
        Asset,
        AssetScheme,
        // AssetTransferAddress
        AssetTransferAddress,
        // AssetAgent
        PubkeyAssetAgent,
        // KeyStore
        MemoryKeyStore,
    };
}
