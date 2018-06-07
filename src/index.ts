import { H160, H512, SignedParcel, H256, Parcel, U256, Invoice, Asset, AssetScheme, Block } from "./primitives/index";

/**
 * @hidden
 */
const jayson = require("jayson");

/**
 * @hidden
 */
interface RpcRequest {
    name: string;
    toRpcParameter: (...params: any[]) => any[];
    fromRpcResult: (result: any) => any;
}

export class SDK {
    private client: any;

    constructor(httpUrl: string) {
        this.client = jayson.client.http(httpUrl);
    }

    private createRpcRequest = (request: RpcRequest) => {
        return (...params: any[]) => {
            const { name, toRpcParameter, fromRpcResult } = request;
            return new Promise<any>((resolve, reject) => {
                this.client.request(name, toRpcParameter(...params), (err: any, res: any) => {
                    if (err) {
                        return reject(err);
                    } else if (res.error) {
                        return reject(res.error);
                    }
                    resolve(fromRpcResult(res.result));
                });
            });
        };
    }

    /**
     * Sends ping to check whether CodeChain's RPC server is responding or not.
     * @returns String "pong"
     */
    ping(): Promise<string> {
        return this.createRpcRequest({
            name: "ping",
            toRpcParameter: () => [],
            fromRpcResult: result => result
        })();
    }

    /**
     * Sends SignedParcel to CodeChain's network.
     * @param parcel SignedParcel
     * @returns SignedParcel's hash.
     */
    sendSignedParcel(parcel: SignedParcel): Promise<H256> {
        return this.createRpcRequest({
            name: "chain_sendSignedParcel",
            toRpcParameter: (p: SignedParcel) => {
                const bytes = Array.from(p.rlpBytes()).map(byte => byte < 0x10 ? `0${byte.toString(16)}` : byte.toString(16)).join("");
                return [`0x${bytes}`];
            },
            fromRpcResult: result => new H256(result)
        })(parcel);
    }

    /**
     * Gets SignedParcel of given hash. Else returns null.
     * @param hash SignedParcel's hash
     * @returns SignedParcel, or null when SignedParcel was not found.
     */
    getParcel(hash: H256): Promise<SignedParcel | null> {
        return this.createRpcRequest({
            name: "chain_getParcel",
            toRpcParameter: (hash: H256) => [`0x${hash.value}`],
            fromRpcResult: result => result === null ? null : SignedParcel.fromJSON(result)
        })(hash);
    }

    // FIXME: timeout not implemented
    /**
     * Gets invoices of given parcel.
     * @param parcelHash The parcel hash of which to get the corresponding parcel of.
     * @param _timeout Indicating milliseconds to wait the parcel to be confirmed.
     * @returns List of invoice, or null when no such parcel exists.
     */
    getParcelInvoices(parcelHash: H256, _timeout?: number): Promise<Invoice[] | null> {
        return this.createRpcRequest({
            name: "chain_getParcelInvoices",
            toRpcParameter: (parcelHash: H256) => [`0x${parcelHash.value}`],
            fromRpcResult: result => result === null ? null : result.map((invoice: any) => Invoice.fromJSON(invoice))
        })(parcelHash, _timeout);
    }

    /**
     * Gets the regular key of an account of given address, recorded in the block of given blockNumber. If blockNumber is not given, then returns the regular key in the most recent block.
     * @param address An account address
     * @param blockNumber The specific block number to get account's regular key at given address.
     * @returns The regular key of account at specified block, or null when address was not found.
     */
    getRegularKey(address: H160, blockNumber?: number): Promise<H512 | null> {
        return this.createRpcRequest({
            name: "chain_getRegularKey",
            toRpcParameter: () => [`0x${address.value}`, blockNumber || null],
            fromRpcResult: result => result === null ? null : new H512(result)
        })(address, blockNumber);
    }

    // FIXME: Implement timeout
    /**
     * Gets invoice of a transaction of given hash.
     * @param txhash The transaction hash of which to get the corresponding transaction of.
     * @returns Invoice, or null when transaction of given hash not exists.
     */
    getTransactionInvoice(txhash: H256): Promise<Invoice | null> {
        return this.createRpcRequest({
            name: "chain_getTransactionInvoice",
            toRpcParameter: () => [`0x${txhash.value}`],
            fromRpcResult: result => result === null ? null : Invoice.fromJSON(result)
        })(txhash);
    }

    /**
     * Gets balance of an account of given address, recorded in the block of given blockNumber. If blockNumber is not given, then returns balance recorded in the most recent block.
     * @param address An account address
     * @param blockNumber The specific block number to get account's balance at given address.
     * @returns Balance of account at specified block, or null when address was not found.
     */
    getBalance(address: H160, blockNumber?: number): Promise<U256 | null> {
        return this.createRpcRequest({
            name: "chain_getBalance",
            toRpcParameter: (address: H160, blockNumber?: number) => [`0x${address.value}`, blockNumber],
            fromRpcResult: result => result ? new U256(result) : null
        })(address, blockNumber);
    }

    /**
     * Gets nonce of an account of given address, recorded in the block of given blockNumber. If blockNumber is not given, then returns nonce recorded in the most recent block.
     * @param address An account address
     * @param blockNumber The specific block number to get account's nonce at given address.
     * @returns Nonce of account at specified block, or null when address was not found.
     */
    getNonce(address: H160, blockNumber?: number): Promise<U256 | null> {
        return this.createRpcRequest({
            name: "chain_getNonce",
            toRpcParameter: (address: H160, blockNumber?: number) => [`0x${address.value}`, blockNumber],
            fromRpcResult: result => result ? new U256(result) : null
        })(address, blockNumber);
    }

    /**
     * Gets number of the latest block.
     * @returns Number of the latest block.
     */
    getBlockNumber(): Promise<number> {
        return this.createRpcRequest({
            name: "chain_getBlockNumber",
            toRpcParameter: () => [],
            fromRpcResult: result => result
        })();
    }

    /**
     * Gets block hash of given blockNumber.
     * @param blockNumber The block number of which to get the block hash of.
     * @returns BlockHash, if block exists. Else, returns null.
     */
    getBlockHash(blockNumber: number): Promise<H256 | null> {
        return this.createRpcRequest({
            name: "chain_getBlockHash",
            toRpcParameter: (blockNumber: number) => [blockNumber],
            fromRpcResult: result => result ? new H256(result) : null
        })(blockNumber);
    }

    /**
     * Gets block of given block hash.
     * @param hash The block hash of which to get the block of
     * @returns Block, if block exists. Else, returns null.
     */
    getBlock(hash: H256): Promise<Block | null> {
        return this.createRpcRequest({
            name: "chain_getBlockByHash",
            toRpcParameter: (hash) => [`0x${hash.value}`],
            fromRpcResult: result => result === null ? null : Block.fromJSON(result)
        })(hash);
    }

    // FIXME: receive asset type instead of txhash. Need to change codechain also.
    /**
     * Gets asset scheme of given hash of AssetMintTransaction.
     * @param txhash The tx hash of AssetMintTransaction.
     * @returns AssetScheme, if asset scheme exists. Else, returns null.
     */
    getAssetScheme(txhash: H256): Promise<AssetScheme | null> {
        return this.createRpcRequest({
            name: "chain_getAssetScheme",
            toRpcParameter: (txhash: H256) => [`0x${txhash.value}`],
            fromRpcResult: result => {
                if (!result) {
                    return null;
                }
                return AssetScheme.fromJSON(result);
            }
        })(txhash);
    }

    /**
     * Gets asset of given transaction hash and index.
     * @param txhash The tx hash of AssetMintTransaction or AssetTransferTransaction.
     * @param index The index of output in the transaction.
     * @returns Asset, if asset exists, Else, returns null.
     */
    getAsset(txhash: H256, index: number): Promise<AssetScheme | null> {
        return this.createRpcRequest({
            name: "chain_getAsset",
            toRpcParameter: (txhash: H256, index: number) => [`0x${txhash.value}`, index],
            fromRpcResult: result => {
                if (!result) {
                    return null;
                }
                return Asset.fromJSON(result);
            }
        })(txhash, index);
    }

    /**
     * Gets pending parcels.
     * @returns List of SignedParcel, with each parcel has null for blockNumber/blockHash/parcelIndex.
     */
    getPendingParcels(): Promise<SignedParcel[]> {
        return this.createRpcRequest({
            name: "chain_getPendingParcels",
            toRpcParameter: () => [],
            fromRpcResult: result => result.map((p: any) => SignedParcel.fromJSON(p))
        })();
    }
}

export * from "./primitives/";
export * from "./primitives/transaction";
export * from "./utils";
