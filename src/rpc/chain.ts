import { Rpc } from ".";
import { H256 } from "../core/H256";
import { SignedParcel } from "../core/SignedParcel";
import { H160 } from "../core/H160";
import { U256 } from "../core/U256";
import { AssetScheme } from "../core/AssetScheme";
import { Block } from "../core/Block";
import { Asset } from "../core/Asset";
import { Invoice } from "../core/Invoice";
import { H512 } from "../core/H512";
import { Parcel } from "../core/Parcel";

export class ChainRpc {
    private rpc: Rpc;
    private parcelSigner?: string;
    private parcelFee?: number;

    /**
     * @hidden
     */
    constructor(rpc: Rpc, options: { parcelSigner?: string, parcelFee?: number }) {
        const { parcelSigner, parcelFee } = options;
        this.rpc = rpc;
        this.parcelSigner = parcelSigner;
        this.parcelFee = parcelFee;
    }

    /**
     * Sends SignedParcel to CodeChain's network.
     * @param parcel SignedParcel
     * @returns SignedParcel's hash.
     */
    sendSignedParcel(parcel: SignedParcel): Promise<H256> {
        const bytes = Array.from(parcel.rlpBytes()).map(byte => byte < 0x10 ? `0${byte.toString(16)}` : byte.toString(16)).join("");
        return this.rpc.sendRpcRequest(
            "chain_sendSignedParcel",
            [`0x${bytes}`]
        ).then(result => new H256(result));
    }

    /**
     * Signs a parcel with the given account and sends it to CodeChain's network.
     * @param parcel The parcel to send
     * @param options.account The account to sign the parcel
     * @param options.passphrase The account's passphrase
     * @param options.nonce The nonce of the parcel
     * @param options.fee The fee of the parcel
     * @returns SignedParcel's hash
     * @throws When the given account cannot afford to pay the fee
     * @throws When the given fee is too low
     * @throws When the given nonce does not match
     * @throws When the given account is unknown
     * @throws When the given passphrase does not match
     */
    async sendParcel(parcel: Parcel, options?: {
        account?: H160 | string,
        passphrase?: string,
        nonce?: U256 | string | number,
        fee?: U256 | string | number,
    }): Promise<H256> {
        const {
            account = this.parcelSigner,
            passphrase = undefined,
            fee = this.parcelFee
        } = options || {};
        if (!account) {
            throw "The account to sign the parcel is not specified";
        }
        const { nonce = await this.getNonce(account) } = options || {};
        parcel.setNonce(nonce!);
        if (!fee) {
            throw "The fee of the parcel is not specified";
        }
        parcel.setFee(fee);
        const sig = await this.rpc.account.sign(parcel.hash(), account, passphrase);
        return this.sendSignedParcel(new SignedParcel(parcel, sig));
    }

    /**
     * Gets SignedParcel of given hash. Else returns null.
     * @param hash SignedParcel's hash
     * @returns SignedParcel, or null when SignedParcel was not found.
     */
    getParcel(hash: H256 | string): Promise<SignedParcel | null> {
        return this.rpc.sendRpcRequest(
            "chain_getParcel",
            [`0x${H256.ensure(hash).value}`]
        ).then(result => result === null ? null : SignedParcel.fromJSON(result));
    }

    /**
     * Gets invoices of given parcel.
     * @param parcelHash The parcel hash of which to get the corresponding parcel of.
     * @param options.timeout Indicating milliseconds to wait the parcel to be confirmed.
     * @returns List of invoice, or null when no such parcel exists.
     */
    async getParcelInvoice(parcelHash: H256 | string, options: { timeout?: number } = {}): Promise<Invoice[] | Invoice | null> {
        const attemptToGet = async () => {
            return this.rpc.sendRpcRequest(
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
        const { timeout } = options;
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
        return this.rpc.sendRpcRequest(
            "chain_getRegularKey",
            [`0x${H160.ensure(address).value}`, blockNumber || null]
        ).then(result => result === null ? null : new H512(result));
    }

    /**
     * Gets invoice of a transaction of given hash.
     * @param txhash The transaction hash of which to get the corresponding transaction of.
     * @param options.timeout Indicating milliseconds to wait the transaction to be confirmed.
     * @returns Invoice, or null when transaction of given hash not exists.
     */
    async getTransactionInvoice(txhash: H256 | string, options: { timeout?: number } = {}): Promise<Invoice | null> {
        const attemptToGet = async () => {
            return this.rpc.sendRpcRequest(
                "chain_getTransactionInvoice",
                [`0x${H256.ensure(txhash).value}`]
            ).then(result => result === null ? null : Invoice.fromJSON(result));
        };
        const startTime = Date.now();
        const { timeout } = options;
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
        return this.rpc.sendRpcRequest(
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
        return this.rpc.sendRpcRequest(
            "chain_getNonce",
            [`0x${H160.ensure(address).value}`, blockNumber]
        ).then(result => result ? new U256(result) : null);
    }

    /**
     * Gets number of the latest block.
     * @returns Number of the latest block.
     */
    getBestBlockNumber(): Promise<number> {
        return this.rpc.sendRpcRequest(
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
        return this.rpc.sendRpcRequest(
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
            return this.rpc.sendRpcRequest(
                "chain_getBlockByHash",
                [`0x${H256.ensure(hashOrNumber).value}`]
            ).then(result => result === null ? null : Block.fromJSON(result));
        } else {
            return this.rpc.sendRpcRequest(
                "chain_getBlockByNumber",
                [hashOrNumber]
            ).then(result => result === null ? null : Block.fromJSON(result));
        }
    }

    /**
     * Gets asset scheme of given hash of AssetMintTransaction.
     * @param txhash The tx hash of AssetMintTransaction.
     * @param shardId The shard id of Asset Scheme.
     * @returns AssetScheme, if asset scheme exists. Else, returns null.
     */
    getAssetSchemeByHash(txhash: H256 | string, shardId: number): Promise<AssetScheme | null> {
        return this.rpc.sendRpcRequest(
            "chain_getAssetSchemeByHash",
            [`0x${H256.ensure(txhash).value}`, shardId]
        ).then(result => result === null ? null : AssetScheme.fromJSON(result));
    }

    /**
     * Gets asset scheme of asset type
     * @param assetType The type of Asset.
     * @returns AssetScheme, if asset scheme exists. Else, returns null.
     */
    getAssetSchemeByType(assetType: H256 | string): Promise<AssetScheme | null> {
        return this.rpc.sendRpcRequest(
            "chain_getAssetSchemeByType",
            [`0x${H256.ensure(assetType).value}`]
        ).then(result => result === null ? null : AssetScheme.fromJSON(result));
    }

    /**
     * Gets asset of given transaction hash and index.
     * @param txhash The tx hash of AssetMintTransaction or AssetTransferTransaction.
     * @param index The index of output in the transaction.
     * @param blockNumber The specific block number to get the asset from
     * @returns Asset, if asset exists, Else, returns null.
     */
    getAsset(txhash: H256 | string, index: number, blockNumber?: number): Promise<Asset | null> {
        return this.rpc.sendRpcRequest(
            "chain_getAsset",
            [`0x${H256.ensure(txhash).value}`, index, blockNumber]
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
        return this.rpc.sendRpcRequest(
            "chain_getPendingParcels",
            []
        ).then(result => result.map((p: any) => SignedParcel.fromJSON(p)));
    }

}
