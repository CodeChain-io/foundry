import { PlatformAddress } from "codechain-primitives";
import { Rpc } from ".";
import { Asset } from "../core/Asset";
import { AssetScheme } from "../core/AssetScheme";
import { Block } from "../core/Block";
import { H256 } from "../core/H256";
import { H512 } from "../core/H512";
import { Invoice } from "../core/Invoice";
import { Parcel } from "../core/Parcel";
import { SignedParcel } from "../core/SignedParcel";
import { Transaction } from "../core/transaction/Transaction";
import { NetworkId } from "../core/types";
import { U256 } from "../core/U256";
export declare class ChainRpc {
    private rpc;
    private parcelSigner?;
    private parcelFee?;
    /**
     * @hidden
     */
    constructor(rpc: Rpc, options: {
        parcelSigner?: string;
        parcelFee?: number;
    });
    /**
     * Sends SignedParcel to CodeChain's network.
     * @param parcel SignedParcel
     * @returns SignedParcel's hash.
     */
    sendSignedParcel(parcel: SignedParcel): Promise<H256>;
    /**
     * Signs a parcel with the given account and sends it to CodeChain's network.
     * @param parcel The parcel to send
     * @param options.account The account to sign the parcel
     * @param options.passphrase The account's passphrase
     * @param options.seq The seq of the parcel
     * @param options.fee The fee of the parcel
     * @returns SignedParcel's hash
     * @throws When the given account cannot afford to pay the fee
     * @throws When the given fee is too low
     * @throws When the given seq does not match
     * @throws When the given account is unknown
     * @throws When the given passphrase does not match
     */
    sendParcel(parcel: Parcel, options?: {
        account?: PlatformAddress | string;
        passphrase?: string;
        seq?: U256 | string | number;
        fee?: U256 | string | number;
    }): Promise<H256>;
    /**
     * Gets SignedParcel of given hash. Else returns null.
     * @param hash SignedParcel's hash
     * @returns SignedParcel, or null when SignedParcel was not found.
     */
    getParcel(hash: H256 | string): Promise<SignedParcel | null>;
    /**
     * Gets invoices of given parcel.
     * @param parcelHash The parcel hash of which to get the corresponding parcel of.
     * @param options.timeout Indicating milliseconds to wait the parcel to be confirmed.
     * @returns List of invoice, or null when no such parcel exists.
     */
    getParcelInvoice(parcelHash: H256 | string, options?: {
        timeout?: number;
    }): Promise<Invoice | null>;
    /**
     * Gets the regular key of an account of given address, recorded in the block of given blockNumber. If blockNumber is not given, then returns the regular key in the most recent block.
     * @param address An account address
     * @param blockNumber The specific block number to get account's regular key at given address.
     * @returns The regular key of account at specified block.
     */
    getRegularKey(address: PlatformAddress | string, blockNumber?: number): Promise<H512 | null>;
    /**
     * Gets the owner of a regular key, recorded in the block of given blockNumber.
     * @param regularKey A regular key.
     * @param blockNumber A block number.
     * @return The platform address that can use the regular key at the specified block.
     */
    getRegularKeyOwner(regularKey: H512 | string, blockNumber?: number): Promise<PlatformAddress | null>;
    /**
     * Gets a transaction of given hash.
     * @param txhash The transaction hash of which to get the corresponding transaction of.
     * @returns A transaction, or null when transaction of given hash not exists.
     */
    getTransaction(txhash: H256 | string): Promise<Transaction | null>;
    /**
     * Gets invoice of a transaction of given hash.
     * @param txhash The transaction hash of which to get the corresponding transaction of.
     * @param options.timeout Indicating milliseconds to wait the transaction to be confirmed.
     * @returns Invoice, or null when transaction of given hash not exists.
     */
    getTransactionInvoices(txhash: H256 | string, options?: {
        timeout?: number;
    }): Promise<Invoice[]>;
    /**
     * Gets balance of an account of given address, recorded in the block of given blockNumber. If blockNumber is not given, then returns balance recorded in the most recent block.
     * @param address An account address
     * @param blockNumber The specific block number to get account's balance at given address.
     * @returns Balance of account at the specified block, or null if no such block exists.
     */
    getBalance(address: PlatformAddress | string, blockNumber?: number): Promise<U256>;
    /**
     * Gets seq of an account of given address, recorded in the block of given blockNumber. If blockNumber is not given, then returns seq recorded in the most recent block.
     * @param address An account address
     * @param blockNumber The specific block number to get account's seq at given address.
     * @returns Seq of account at the specified block, or null if no such block exists.
     */
    getSeq(address: PlatformAddress | string, blockNumber?: number): Promise<U256>;
    /**
     * Gets number of the latest block.
     * @returns Number of the latest block.
     */
    getBestBlockNumber(): Promise<number>;
    /**
     * Gets block hash of given blockNumber.
     * @param blockNumber The block number of which to get the block hash of.
     * @returns BlockHash, if block exists. Else, returns null.
     */
    getBlockHash(blockNumber: number): Promise<H256 | null>;
    /**
     * Gets block of given block hash.
     * @param hashOrNumber The block hash or number of which to get the block of
     * @returns Block, if block exists. Else, returns null.
     */
    getBlock(hashOrNumber: H256 | string | number): Promise<Block | null>;
    /**
     * Gets asset scheme of given hash of AssetMintTransaction.
     * @param txhash The tx hash of AssetMintTransaction.
     * @param shardId The shard id of Asset Scheme.
     * @returns AssetScheme, if asset scheme exists. Else, returns null.
     */
    getAssetSchemeByHash(txhash: H256 | string, shardId: number): Promise<AssetScheme | null>;
    /**
     * Gets asset scheme of asset type
     * @param assetType The type of Asset.
     * @returns AssetScheme, if asset scheme exists. Else, returns null.
     */
    getAssetSchemeByType(assetType: H256 | string): Promise<AssetScheme | null>;
    /**
     * Gets asset of given transaction hash and index.
     * @param txhash The tx hash of AssetMintTransaction or AssetTransferTransaction.
     * @param index The index of output in the transaction.
     * @param blockNumber The specific block number to get the asset from
     * @returns Asset, if asset exists, Else, returns null.
     */
    getAsset(txhash: H256 | string, index: number, blockNumber?: number): Promise<Asset | null>;
    /**
     * Checks whether an asset is spent or not.
     * @param txhash The tx hash of AssetMintTransaction or AssetTransferTransaction.
     * @param index The index of output in the transaction.
     * @param shardId The shard id of an Asset.
     * @param blockNumber The specific block number to get the asset from.
     * @returns True, if the asset is spent. False, if the asset is not spent. Null, if no such asset exists.
     */
    isAssetSpent(txhash: H256 | string, index: number, shardId: number, blockNumber?: number): Promise<boolean | null>;
    /**
     * Gets pending parcels.
     * @returns List of SignedParcel, with each parcel has null for blockNumber/blockHash/parcelIndex.
     */
    getPendingParcels(): Promise<SignedParcel[]>;
    /**
     * Gets the network ID of the node.
     * @returns A network ID, e.g. "tc".
     */
    getNetworkId(): Promise<NetworkId>;
}
