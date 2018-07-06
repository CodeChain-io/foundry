import { H160, H512, SignedParcel, H256, Parcel, U256, Invoice, Asset, AssetScheme, Block } from "./primitives/index";
import { getTransactionFromJSON, Transaction, AssetMintTransaction, AssetTransferTransaction, AssetTransferInput, AssetOutPoint, AssetTransferOutput } from "./primitives/transaction";
import { blake256, blake256WithKey, ripemd160, signEcdsa, getPublicFromPrivate, getAccountIdFromPrivate, verifyEcdsa, recoverEcdsa, generatePrivateKey } from "./utils";
import { AssetTransferAddress } from "./AssetTransferAddress";
import { PlatformAddress } from "./PlatformAddress";
import { PubkeyAssetAgent, KeyStore } from "./signer/PubkeyAssetAgent";
import { MemoryKeyStore } from "./signer/MemoryKeyStore";
import { Payment, SetRegularKey, ChangeShardState, CreateShard } from "./primitives/Parcel";
import { AssetAgent } from "./primitives/Asset";
import { Rpc } from "./rpc";

/**
 * @hidden
 */
export type ParcelParams = {
    nonce: U256 | number | string;
    fee: U256 | number | string;
};

class SDK {
    private networkId: number;
    private keyStore: KeyStore;
    private assetAgent: AssetAgent;

    public rpc: Rpc;

    /**
     * @param params.server HTTP RPC server address
     * @param params.networkId The network id of CodeChain. The default value is 0x11 (solo consensus)
     */
    constructor(params: { server: string, networkId?: number }) {
        const { server, networkId = 0x11 } = params;
        this.networkId = networkId;
        this.keyStore = new MemoryKeyStore();
        this.assetAgent = new PubkeyAssetAgent({ keyStore: this.keyStore });

        this.rpc = new Rpc({ server });
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

    // Primitives
    static SDK = SDK;
    static H160 = H160;
    static H256 = H256;
    static H512 = H512;
    static U256 = U256;
    static Parcel = Parcel;
    static SignedParcel = SignedParcel;
    static Invoice = Invoice;
    static Asset = Asset;
    static AssetScheme = AssetScheme;
    static Block = Block;

    // Action
    static Payment = Payment;
    static SetRegularKey = SetRegularKey;
    static ChangeShardState = ChangeShardState;

    // Address
    static AssetTransferAddress = AssetTransferAddress;
    static PlatformAddress = PlatformAddress;

    static PubkeyAssetAgent = PubkeyAssetAgent;
    static MemoryKeyStore = MemoryKeyStore;

    // Transactions
    static AssetMintTransaction = AssetMintTransaction;
    static AssetTransferTransaction = AssetTransferTransaction;
    static AssetTransferInput = AssetTransferInput;
    static AssetOutPoint = AssetOutPoint;
    static AssetTransferOutput = AssetTransferOutput;
    static getTransactionFromJSON = getTransactionFromJSON;

    // Utils
    static blake256 = blake256;
    static blake256WithKey = blake256WithKey;
    static ripemd160 = ripemd160;
    static signEcdsa = signEcdsa;
    static verifyEcdsa = verifyEcdsa;
    static recoverEcdsa = recoverEcdsa;
    static generatePrivateKey = generatePrivateKey;
    static getAccountIdFromPrivate = getAccountIdFromPrivate;
    static getPublicFromPrivate = getPublicFromPrivate;

}

export { SDK };
export { H160, H512, SignedParcel, H256, Parcel, U256, Invoice, Asset, AssetScheme, Block };
export { getTransactionFromJSON, Transaction, AssetMintTransaction, AssetTransferTransaction, AssetTransferInput, AssetOutPoint, AssetTransferOutput };
export { blake256, blake256WithKey, ripemd160, signEcdsa, generatePrivateKey, getPublicFromPrivate, getAccountIdFromPrivate };
export { PubkeyAssetAgent };
export { MemoryKeyStore };
export { PlatformAddress, AssetTransferAddress };
export { ChangeShardState, Payment, SetRegularKey };

module.exports = SDK;
