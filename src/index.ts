import { H160, H512, SignedParcel, H256, Parcel, U256, Invoice, Asset, AssetScheme, Block } from "./primitives/index";
import { getTransactionFromJSON, Transaction, PaymentTransaction, SetRegularKeyTransaction, AssetMintTransaction, AssetTransferTransaction, AssetTransferInput, AssetOutPoint, AssetTransferOutput } from "./primitives/transaction";
import { blake256, blake256WithKey, ripemd160, signEcdsa, privateKeyToPublic, privateKeyToAddress, verifyEcdsa, recoverPublic } from "./utils";

/**
 * @hidden
 */
const jayson = require("jayson");

class SDK {
    private client: any;

    constructor(httpUrl: string) {
        this.client = jayson.client.http(httpUrl);
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
    getParcel(hash: H256): Promise<SignedParcel | null> {
        return this.sendRpcRequest(
            "chain_getParcel",
            [`0x${hash.value}`]
        ).then(result => result === null ? null : SignedParcel.fromJSON(result));
    }

    // FIXME: timeout not implemented
    /**
     * Gets invoices of given parcel.
     * @param parcelHash The parcel hash of which to get the corresponding parcel of.
     * @param _timeout Indicating milliseconds to wait the parcel to be confirmed.
     * @returns List of invoice, or null when no such parcel exists.
     */
    getParcelInvoice(parcelHash: H256, _timeout?: number): Promise<Invoice[] | Invoice | null> {
        return this.sendRpcRequest(
            "chain_getParcelInvoice",
            [`0x${parcelHash.value}`]
        ).then(result => {
            if (result === null) {
                return null;
            }
            if (Array.isArray(result)) {
                return result.map((invoice: any) => Invoice.fromJSON(invoice));
            }
            return Invoice.fromJSON(result);
        });
    }

    /**
     * Gets the regular key of an account of given address, recorded in the block of given blockNumber. If blockNumber is not given, then returns the regular key in the most recent block.
     * @param address An account address
     * @param blockNumber The specific block number to get account's regular key at given address.
     * @returns The regular key of account at specified block, or null when address was not found.
     */
    getRegularKey(address: H160, blockNumber?: number): Promise<H512 | null> {
        return this.sendRpcRequest(
            "chain_getRegularKey",
            [`0x${address.value}`, blockNumber || null]
        ).then(result => result === null ? null : new H512(result));
    }

    // FIXME: Implement timeout
    /**
     * Gets invoice of a transaction of given hash.
     * @param txhash The transaction hash of which to get the corresponding transaction of.
     * @returns Invoice, or null when transaction of given hash not exists.
     */
    getTransactionInvoice(txhash: H256): Promise<Invoice | null> {
        return this.sendRpcRequest(
            "chain_getTransactionInvoice",
            [`0x${txhash.value}`]
        ).then(result => result === null ? null : Invoice.fromJSON(result));
    }

    /**
     * Gets balance of an account of given address, recorded in the block of given blockNumber. If blockNumber is not given, then returns balance recorded in the most recent block.
     * @param address An account address
     * @param blockNumber The specific block number to get account's balance at given address.
     * @returns Balance of account at specified block, or null when address was not found.
     */
    getBalance(address: H160, blockNumber?: number): Promise<U256 | null> {
        return this.sendRpcRequest(
            "chain_getBalance",
            [`0x${address.value}`, blockNumber]
        ).then(result => result ? new U256(result) : null);
    }

    /**
     * Gets nonce of an account of given address, recorded in the block of given blockNumber. If blockNumber is not given, then returns nonce recorded in the most recent block.
     * @param address An account address
     * @param blockNumber The specific block number to get account's nonce at given address.
     * @returns Nonce of account at specified block, or null when address was not found.
     */
    getNonce(address: H160, blockNumber?: number): Promise<U256 | null> {
        return this.sendRpcRequest(
            "chain_getNonce",
            [`0x${address.value}`, blockNumber]
        ).then(result => result ? new U256(result) : null);
    }

    /**
     * Gets number of the latest block.
     * @returns Number of the latest block.
     */
    getBlockNumber(): Promise<number> {
        return this.sendRpcRequest(
            "chain_getBlockNumber",
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
    getBlock(hashOrNumber: H256 | number): Promise<Block | null> {
        if (hashOrNumber instanceof H256) {
            return this.sendRpcRequest(
                "chain_getBlockByHash",
                [`0x${hashOrNumber.value}`]
            ).then(result => result === null ? null : Block.fromJSON(result));
        } else {
            // FIXME: use chain_getBlockByNumber when it's implemented
            return this.getBlockHash(hashOrNumber).then(hash => {
                if (hash === null) {
                    return null;
                }
                return this.getBlock(hash);
            });
        }
    }

    // FIXME: receive asset type instead of txhash. Need to change codechain also.
    /**
     * Gets asset scheme of given hash of AssetMintTransaction.
     * @param txhash The tx hash of AssetMintTransaction.
     * @returns AssetScheme, if asset scheme exists. Else, returns null.
     */
    getAssetScheme(txhash: H256): Promise<AssetScheme | null> {
        return this.sendRpcRequest(
            "chain_getAssetScheme",
            [`0x${txhash.value}`]
        ).then(result => result === null ? null : AssetScheme.fromJSON(result));
    }

    /**
     * Gets asset of given transaction hash and index.
     * @param txhash The tx hash of AssetMintTransaction or AssetTransferTransaction.
     * @param index The index of output in the transaction.
     * @returns Asset, if asset exists, Else, returns null.
     */
    getAsset(txhash: H256, index: number): Promise<Asset | null> {
        return this.sendRpcRequest(
            "chain_getAsset",
            [`0x${txhash.value}`, index]
        ).then(result => result === null ? null : Asset.fromJSON(result));
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

    // Transactions
    static PaymentTransaction = PaymentTransaction;
    static SetRegularKeyTransaction = SetRegularKeyTransaction;
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
    static privateKeyToAddress = privateKeyToAddress;
    static privateKeyToPublic = privateKeyToPublic;
}

export { SDK };
export { H160, H512, SignedParcel, H256, Parcel, U256, Invoice, Asset, AssetScheme, Block };
export { getTransactionFromJSON, Transaction, PaymentTransaction, SetRegularKeyTransaction, AssetMintTransaction, AssetTransferTransaction, AssetTransferInput, AssetOutPoint, AssetTransferOutput };
export { blake256, blake256WithKey, ripemd160, signEcdsa, privateKeyToPublic, privateKeyToAddress };

module.exports = SDK;
