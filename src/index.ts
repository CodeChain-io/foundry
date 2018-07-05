import { H160, H512, SignedParcel, H256, Parcel, U256, Invoice, Asset, AssetScheme, Block } from "./primitives/index";
import { getTransactionFromJSON, Transaction, AssetMintTransaction, AssetTransferTransaction, AssetTransferInput, AssetOutPoint, AssetTransferOutput } from "./primitives/transaction";
import { blake256, blake256WithKey, ripemd160, signEcdsa, privateKeyToPublic, privateKeyToAddress, verifyEcdsa, recoverPublic, generatePrivateKey } from "./utils";
import { AssetTransferAddress } from "./AssetTransferAddress";
import { PlatformAddress } from "./PlatformAddress";
import { PubkeyAssetAgent, KeyStore } from "./signer/PubkeyAssetAgent";
import { MemoryKeyStore } from "./signer/MemoryKeyStore";
import { Payment, SetRegularKey, ChangeShardState, CreateShard } from "./primitives/Parcel";
import { AssetAgent } from "./primitives/Asset";

import fetch from "node-fetch";

/**
 * @hidden
 */
const jaysonBrowserClient = require("jayson/lib/client/browser");
/**
 * @hidden
 */
export type ParcelParams = {
    nonce: U256 | number | string;
    fee: U256 | number | string;
};

class SDK {
    private client: any;
    private networkId: number;
    private keyStore: KeyStore;
    private assetAgent: AssetAgent;

    /**
     * @param params.server HTTP RPC server address
     * @param params.networkId The network id of CodeChain. The default value is 0x11 (solo consensus)
     */
    constructor(params: { server: string, networkId?: number }) {
        const { server, networkId = 0x11 } = params;
        this.client = jaysonBrowserClient((request: any, callback: any) => {
            fetch(server, {
                method: "POST",
                body: request,
                headers: {
                    "Content-Type": "application/json"
                }
            }).then(res => {
                return res.text();
            }).then(text => {
                return callback(null, text);
            }).catch(err => {
                return callback(err);
            });
        });
        this.networkId = networkId;
        this.keyStore = new MemoryKeyStore();
        this.assetAgent = new PubkeyAssetAgent({ keyStore: this.keyStore });
    }

    private sendRpcRequest = (name: string, params: any[]) => {
        return new Promise<any>((resolve, reject) => {
            this.client.request(name, params, (err: any, res: any) => {
                if (err) {
                    return reject(err);
                } else if (res.error) {
                    return reject(res.error);
                }
                resolve(res.result);
            });
        });
    }

    /**
     * Sends ping to check whether CodeChain's RPC server is responding or not.
     * @returns String "pong"
     */
    ping(): Promise<string> {
        return this.sendRpcRequest(
            "ping",
            []
        );
    }

    /**
     * Gets the version of CodeChain node.
     * @returns The version of CodeChain node (e.g. 0.1.0)
     */
    getNodeVersion(): Promise<string> {
        return this.sendRpcRequest("version", []);
    }

    /**
     * Sends SignedParcel to CodeChain's network.
     * @param parcel SignedParcel
     * @returns SignedParcel's hash.
     */
    sendSignedParcel(parcel: SignedParcel): Promise<H256> {
        const bytes = Array.from(parcel.rlpBytes()).map(byte => byte < 0x10 ? `0${byte.toString(16)}` : byte.toString(16)).join("");
        return this.sendRpcRequest(
            "chain_sendSignedParcel",
            [`0x${bytes}`]
        ).then(result => new H256(result));
    }

    /**
     * Gets SignedParcel of given hash. Else returns null.
     * @param hash SignedParcel's hash
     * @returns SignedParcel, or null when SignedParcel was not found.
     */
    getParcel(hash: H256 | string): Promise<SignedParcel | null> {
        return this.sendRpcRequest(
            "chain_getParcel",
            [`0x${H256.ensure(hash).value}`]
        ).then(result => result === null ? null : SignedParcel.fromJSON(result));
    }

    /**
     * Gets invoices of given parcel.
     * @param parcelHash The parcel hash of which to get the corresponding parcel of.
     * @param timeout Indicating milliseconds to wait the parcel to be confirmed.
     * @returns List of invoice, or null when no such parcel exists.
     */
    async getParcelInvoice(parcelHash: H256 | string, timeout?: number): Promise<Invoice[] | Invoice | null> {
        const attemptToGet = async () => {
            return this.sendRpcRequest(
                "chain_getParcelInvoice",
                [`0x${H256.ensure(parcelHash).value}`]
            ).then(result => {
                if (result === null) {
                    return null;
                }
                if (Array.isArray(result)) {
                    return result.map((invoice: any) => Invoice.fromJSON(invoice));
                }
                return Invoice.fromJSON(result);
            });
        };
        const startTime = Date.now();
        let result = await attemptToGet();
        while (result === null && timeout !== undefined && Date.now() - startTime < timeout) {
            await new Promise(resolve => setTimeout(resolve, 1000));
            result = await attemptToGet();
        }
        return result;
    }

    /**
     * Gets the regular key of an account of given address, recorded in the block of given blockNumber. If blockNumber is not given, then returns the regular key in the most recent block.
     * @param address An account address
     * @param blockNumber The specific block number to get account's regular key at given address.
     * @returns The regular key of account at specified block, or null when address was not found.
     */
    getRegularKey(address: H160 | string, blockNumber?: number): Promise<H512 | null> {
        return this.sendRpcRequest(
            "chain_getRegularKey",
            [`0x${H160.ensure(address).value}`, blockNumber || null]
        ).then(result => result === null ? null : new H512(result));
    }

    /**
     * Gets invoice of a transaction of given hash.
     * @param txhash The transaction hash of which to get the corresponding transaction of.
     * @param timeout Indicating milliseconds to wait the transaction to be confirmed.
     * @returns Invoice, or null when transaction of given hash not exists.
     */
    async getTransactionInvoice(txhash: H256 | string, timeout?: number): Promise<Invoice | null> {
        const attemptToGet = async () => {
            return this.sendRpcRequest(
                "chain_getTransactionInvoice",
                [`0x${H256.ensure(txhash).value}`]
            ).then(result => result === null ? null : Invoice.fromJSON(result));
        };
        const startTime = Date.now();
        let result = await attemptToGet();
        while (result === null && timeout !== undefined && Date.now() - startTime < timeout) {
            await new Promise(resolve => setTimeout(resolve, 1000));
            result = await attemptToGet();
        }
        return result;
    }

    /**
     * Gets balance of an account of given address, recorded in the block of given blockNumber. If blockNumber is not given, then returns balance recorded in the most recent block.
     * @param address An account address
     * @param blockNumber The specific block number to get account's balance at given address.
     * @returns Balance of account at specified block, or null when address was not found.
     */
    getBalance(address: H160 | string, blockNumber?: number): Promise<U256 | null> {
        return this.sendRpcRequest(
            "chain_getBalance",
            [`0x${H160.ensure(address).value}`, blockNumber]
        ).then(result => result ? new U256(result) : null);
    }

    /**
     * Gets nonce of an account of given address, recorded in the block of given blockNumber. If blockNumber is not given, then returns nonce recorded in the most recent block.
     * @param address An account address
     * @param blockNumber The specific block number to get account's nonce at given address.
     * @returns Nonce of account at specified block, or null when address was not found.
     */
    getNonce(address: H160 | string, blockNumber?: number): Promise<U256 | null> {
        return this.sendRpcRequest(
            "chain_getNonce",
            [`0x${H160.ensure(address).value}`, blockNumber]
        ).then(result => result ? new U256(result) : null);
    }

    /**
     * Gets number of the latest block.
     * @returns Number of the latest block.
     */
    getBestBlockNumber(): Promise<number> {
        return this.sendRpcRequest(
            "chain_getBestBlockNumber",
            []
        );
    }

    /**
     * Gets block hash of given blockNumber.
     * @param blockNumber The block number of which to get the block hash of.
     * @returns BlockHash, if block exists. Else, returns null.
     */
    getBlockHash(blockNumber: number): Promise<H256 | null> {
        return this.sendRpcRequest(
            "chain_getBlockHash",
            [blockNumber]
        ).then(result => result ? new H256(result) : null);
    }

    /**
     * Gets block of given block hash.
     * @param hashOrNumber The block hash or number of which to get the block of
     * @returns Block, if block exists. Else, returns null.
     */
    getBlock(hashOrNumber: H256 | string | number): Promise<Block | null> {
        if (hashOrNumber instanceof H256 || typeof hashOrNumber === "string") {
            return this.sendRpcRequest(
                "chain_getBlockByHash",
                [`0x${H256.ensure(hashOrNumber).value}`]
            ).then(result => result === null ? null : Block.fromJSON(result));
        } else {
            return this.sendRpcRequest(
                "chain_getBlockByNumber",
                [hashOrNumber]
            ).then(result => result === null ? null : Block.fromJSON(result));
        }
    }

    /**
     * Gets asset scheme of given hash of AssetMintTransaction.
     * @param txhash The tx hash of AssetMintTransaction.
     * @returns AssetScheme, if asset scheme exists. Else, returns null.
     */
    getAssetScheme(txhash: H256 | string): Promise<AssetScheme | null> {
        return this.sendRpcRequest(
            "chain_getAssetScheme",
            [`0x${H256.ensure(txhash).value}`]
        ).then(result => result === null ? null : AssetScheme.fromJSON(result));
    }

    /**
     * Gets asset of given transaction hash and index.
     * @param txhash The tx hash of AssetMintTransaction or AssetTransferTransaction.
     * @param index The index of output in the transaction.
     * @returns Asset, if asset exists, Else, returns null.
     */
    getAsset(txhash: H256 | string, index: number): Promise<Asset | null> {
        return this.sendRpcRequest(
            "chain_getAsset",
            [`0x${H256.ensure(txhash).value}`, index]
        ).then(result => {
            if (result === null) {
                return null;
            }
            return Asset.fromJSON({
                ...result,
                transactionHash: H256.ensure(txhash).value,
                transactionOutputIndex: index
            });
        });
    }

    /**
     * Gets pending parcels.
     * @returns List of SignedParcel, with each parcel has null for blockNumber/blockHash/parcelIndex.
     */
    getPendingParcels(): Promise<SignedParcel[]> {
        return this.sendRpcRequest(
            "chain_getPendingParcels",
            []
        ).then(result => result.map((p: any) => SignedParcel.fromJSON(p)));
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

    /**
     * Save secret which is used when handshaking with other node,
     * This secret may be exchanged in offline.
     * To use this saved secret, you should call 'net_connect' RPC after this RPC call.
     * @param secret Secret exchanged in offline
     * @param address Node address which RPC server will connect to using secret
     * @param port
     */
    shareSecret(secret: string, address: string, port: number): Promise<null> {
        return this.sendRpcRequest(
            "net_shareSecret",
            [secret, address, port]
        );
    }

    /**
     * Connect to node
     * @param address Node address which to connect
     * @param port
     */
    connect(address: string, port: number): Promise<null> {
        return this.sendRpcRequest(
            "net_connect",
            [address, port]
        );
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
    static recoverPublic = recoverPublic;
    static generatePrivateKey = generatePrivateKey;
    static privateKeyToAddress = privateKeyToAddress;
    static privateKeyToPublic = privateKeyToPublic;
}

export { SDK };
export { H160, H512, SignedParcel, H256, Parcel, U256, Invoice, Asset, AssetScheme, Block };
export { getTransactionFromJSON, Transaction, AssetMintTransaction, AssetTransferTransaction, AssetTransferInput, AssetOutPoint, AssetTransferOutput };
export { blake256, blake256WithKey, ripemd160, signEcdsa, generatePrivateKey, privateKeyToPublic, privateKeyToAddress };
export { PubkeyAssetAgent };
export { MemoryKeyStore };
export { PlatformAddress, AssetTransferAddress };
export { ChangeShardState, Payment, SetRegularKey };

module.exports = SDK;
