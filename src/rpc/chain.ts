import { Rpc } from ".";
import { Asset } from "../core/Asset";
import { AssetScheme } from "../core/AssetScheme";
import { Block } from "../core/Block";
import { H256 } from "../core/H256";
import { H512 } from "../core/H512";
import { Invoice } from "../core/Invoice";
import { Parcel } from "../core/Parcel";
import { SignedParcel } from "../core/SignedParcel";
import {
    getTransactionFromJSON,
    Transaction
} from "../core/transaction/Transaction";
import { NetworkId } from "../core/types";
import { U256 } from "../core/U256";
import { PlatformAddress } from "../key/classes";

export class ChainRpc {
    private rpc: Rpc;
    private parcelSigner?: string;
    private parcelFee?: number;

    /**
     * @hidden
     */
    constructor(
        rpc: Rpc,
        options: { parcelSigner?: string; parcelFee?: number }
    ) {
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
    public sendSignedParcel(parcel: SignedParcel): Promise<H256> {
        if (!(parcel instanceof SignedParcel)) {
            throw Error(
                `Expected the first argument of sendSignedParcel to be SignedParcel but found ${parcel}`
            );
        }
        return new Promise((resolve, reject) => {
            const bytes = Array.from(parcel.rlpBytes())
                .map(
                    byte =>
                        byte < 0x10
                            ? `0${byte.toString(16)}`
                            : byte.toString(16)
                )
                .join("");
            this.rpc
                .sendRpcRequest("chain_sendSignedParcel", [`0x${bytes}`])
                .then(result => {
                    try {
                        resolve(new H256(result));
                    } catch (e) {
                        reject(
                            Error(
                                `Expected sendSignedParcel() to return a value of H256, but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
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
    public async sendParcel(
        parcel: Parcel,
        options?: {
            account?: PlatformAddress | string;
            passphrase?: string;
            nonce?: U256 | string | number;
            fee?: U256 | string | number;
        }
    ): Promise<H256> {
        if (!(parcel instanceof Parcel)) {
            throw Error(
                `Expected the first argument of sendParcel to be a Parcel but found ${parcel}`
            );
        }
        const {
            account = this.parcelSigner,
            passphrase,
            fee = this.parcelFee
        } = options || { passphrase: undefined };
        if (!account) {
            throw Error("The account to sign the parcel is not specified");
        } else if (!PlatformAddress.check(account)) {
            throw Error(
                `Expected account param of sendParcel to be a PlatformAddress value but found ${account}`
            );
        }
        const { nonce = await this.getNonce(account) } = options || {};
        parcel.setNonce(nonce!);
        if (!fee) {
            throw Error("The fee of the parcel is not specified");
        }
        parcel.setFee(fee);
        const address = PlatformAddress.ensure(account);
        const sig = await this.rpc.account.sign(
            parcel.hash(),
            address,
            passphrase
        );
        return this.sendSignedParcel(new SignedParcel(parcel, sig));
    }

    /**
     * Gets SignedParcel of given hash. Else returns null.
     * @param hash SignedParcel's hash
     * @returns SignedParcel, or null when SignedParcel was not found.
     */
    public getParcel(hash: H256 | string): Promise<SignedParcel | null> {
        if (!H256.check(hash)) {
            throw Error(
                `Expected the first argument of getParcel to be an H256 value but found ${hash}`
            );
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getParcel", [
                    `0x${H256.ensure(hash).value}`
                ])
                .then(result => {
                    try {
                        resolve(
                            result === null
                                ? null
                                : SignedParcel.fromJSON(result)
                        );
                    } catch (e) {
                        reject(
                            Error(
                                `Expected chain_getParcel to return either null or JSON of SignedParcel, but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Gets invoices of given parcel.
     * @param parcelHash The parcel hash of which to get the corresponding parcel of.
     * @param options.timeout Indicating milliseconds to wait the parcel to be confirmed.
     * @returns List of invoice, or null when no such parcel exists.
     */
    public async getParcelInvoice(
        parcelHash: H256 | string,
        options: { timeout?: number } = {}
    ): Promise<Invoice[] | Invoice | null> {
        if (!H256.check(parcelHash)) {
            throw Error(
                `Expected the first argument of getParcelInvoice to be an H256 value but found ${parcelHash}`
            );
        }
        const attemptToGet = async () => {
            return this.rpc.sendRpcRequest("chain_getParcelInvoice", [
                `0x${H256.ensure(parcelHash).value}`
            ]);
        };
        const { timeout } = options;
        if (
            timeout !== undefined &&
            (typeof timeout !== "number" || timeout < 0)
        ) {
            throw Error(
                `Expected timeout param of getParcelInvoice to be non-negative number but found ${timeout}`
            );
        }
        const startTime = Date.now();
        let result = await attemptToGet();
        while (
            result === null &&
            timeout !== undefined &&
            Date.now() - startTime < timeout
        ) {
            await new Promise(resolve => setTimeout(resolve, 1000));
            result = await attemptToGet();
        }
        if (result === null) {
            return null;
        }
        try {
            if (Array.isArray(result)) {
                return result.map((invoice: any) => Invoice.fromJSON(invoice));
            }
            return Invoice.fromJSON(result);
        } catch (e) {
            throw Error(
                `Expected chain_getParcelInvoice to return either null or JSON of Invoice, but an error occurred: ${e.toString()}`
            );
        }
    }

    /**
     * Gets the regular key of an account of given address, recorded in the block of given blockNumber. If blockNumber is not given, then returns the regular key in the most recent block.
     * @param address An account address
     * @param blockNumber The specific block number to get account's regular key at given address.
     * @returns The regular key of account at specified block.
     */
    public getRegularKey(
        address: PlatformAddress | string,
        blockNumber?: number
    ): Promise<H512 | null> {
        if (!PlatformAddress.check(address)) {
            throw Error(
                `Expected the first argument of getRegularKey to be a PlatformAddress value but found ${address}`
            );
        }
        if (blockNumber !== undefined && !isNonNegativeInterger(blockNumber)) {
            throw Error(
                `Expected the second argument of getRegularKey to be a number but found ${blockNumber}`
            );
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getRegularKey", [
                    `${PlatformAddress.ensure(address).value}`,
                    blockNumber
                ])
                .then(result => {
                    try {
                        resolve(result === null ? null : new H512(result));
                    } catch (e) {
                        reject(
                            Error(
                                `Expected chain_getRegularKey to return either null or a value of H512, but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Gets the owner of a regular key, recorded in the block of given blockNumber.
     * @param regularKey A regular key.
     * @param blockNumber A block number.
     * @return The platform address that can use the regular key at the specified block.
     */
    public getRegularKeyOwner(
        regularKey: H512 | string,
        blockNumber?: number
    ): Promise<PlatformAddress | null> {
        if (!H512.check(regularKey)) {
            throw Error(
                `Expected the first argument of getRegularKeyOwner to be an H512 value but found ${regularKey}`
            );
        }
        if (blockNumber !== undefined && !isNonNegativeInterger(blockNumber)) {
            throw Error(
                `Expected the second argument of getRegularKeyOwner to be a number but found ${blockNumber}`
            );
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getRegularKeyOwner", [
                    `0x${H512.ensure(regularKey).value}`,
                    blockNumber
                ])
                .then(result => {
                    try {
                        resolve(
                            result === null
                                ? null
                                : PlatformAddress.fromString(result)
                        );
                    } catch (e) {
                        reject(
                            Error(
                                `Expected chain_getRegularKeyOwner to return a PlatformAddress string, but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Gets a transaction of given hash.
     * @param txhash The transaction hash of which to get the corresponding transaction of.
     * @returns A transaction, or null when transaction of given hash not exists.
     */
    public getTransaction(txhash: H256 | string): Promise<Transaction | null> {
        if (!H256.check(txhash)) {
            throw Error(
                `Expected the first argument of getTransaction to be an H256 value but found ${txhash}`
            );
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getTransaction", [
                    `0x${H256.ensure(txhash).value}`
                ])
                .then(result => {
                    try {
                        resolve(
                            result === null
                                ? null
                                : getTransactionFromJSON(result)
                        );
                    } catch (e) {
                        reject(
                            Error(
                                `Expected chain_getTransaction to return either null or JSON of Transaction, but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Gets invoice of a transaction of given hash.
     * @param txhash The transaction hash of which to get the corresponding transaction of.
     * @param options.timeout Indicating milliseconds to wait the transaction to be confirmed.
     * @returns Invoice, or null when transaction of given hash not exists.
     */
    public async getTransactionInvoice(
        txhash: H256 | string,
        options: { timeout?: number } = {}
    ): Promise<Invoice | null> {
        if (!H256.check(txhash)) {
            throw Error(
                `Expected the first argument of getTransactionInvoice to be an H256 value but found ${txhash}`
            );
        }
        const attemptToGet = async () => {
            return this.rpc.sendRpcRequest("chain_getTransactionInvoice", [
                `0x${H256.ensure(txhash).value}`
            ]);
        };
        const startTime = Date.now();
        const { timeout } = options;
        if (
            timeout !== undefined &&
            (typeof timeout !== "number" || timeout < 0)
        ) {
            throw Error(
                `Expected timeout param of getTransactionInvoice to be non-negative number but found ${timeout}`
            );
        }
        let result = await attemptToGet();
        while (
            result === null &&
            timeout !== undefined &&
            Date.now() - startTime < timeout
        ) {
            await new Promise(resolve => setTimeout(resolve, 1000));
            result = await attemptToGet();
        }
        if (result === null) {
            return null;
        }
        try {
            return Invoice.fromJSON(result);
        } catch (e) {
            throw Error(
                `Expected chain_getTransactionInvoice to return either null or JSON of Invoice, but an error occurred: ${e.toString()}`
            );
        }
    }

    /**
     * Gets balance of an account of given address, recorded in the block of given blockNumber. If blockNumber is not given, then returns balance recorded in the most recent block.
     * @param address An account address
     * @param blockNumber The specific block number to get account's balance at given address.
     * @returns Balance of account at the specified block, or null if no such block exists.
     */
    public getBalance(
        address: PlatformAddress | string,
        blockNumber?: number
    ): Promise<U256> {
        if (!PlatformAddress.check(address)) {
            throw Error(
                `Expected the first argument of getBalance to be a PlatformAddress value but found ${address}`
            );
        }
        if (blockNumber !== undefined && !isNonNegativeInterger(blockNumber)) {
            throw Error(
                `Expected the second argument of getBalance to be a number but found ${blockNumber}`
            );
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getBalance", [
                    `${PlatformAddress.ensure(address).value}`,
                    blockNumber
                ])
                .then(result => {
                    try {
                        // FIXME: Need to discuss changing the return type to `U256 | null`. It's a
                        // breaking change.
                        resolve(
                            result === null ? (null as any) : new U256(result)
                        );
                    } catch (e) {
                        reject(
                            Error(
                                `Expected chain_getBalance to return a value of U256, but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Gets nonce of an account of given address, recorded in the block of given blockNumber. If blockNumber is not given, then returns nonce recorded in the most recent block.
     * @param address An account address
     * @param blockNumber The specific block number to get account's nonce at given address.
     * @returns Nonce of account at the specified block, or null if no such block exists.
     */
    public getNonce(
        address: PlatformAddress | string,
        blockNumber?: number
    ): Promise<U256> {
        if (!PlatformAddress.check(address)) {
            throw Error(
                `Expected the first argument of getNonce to be a PlatformAddress value but found ${address}`
            );
        }
        if (blockNumber !== undefined && !isNonNegativeInterger(blockNumber)) {
            throw Error(
                `Expected the second argument of getNonce to be a number but found ${blockNumber}`
            );
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getNonce", [
                    `${PlatformAddress.ensure(address).value}`,
                    blockNumber
                ])
                .then(result => {
                    try {
                        // FIXME: Need to discuss changing the return type to `U256 | null`. It's a
                        // breaking change.
                        resolve(
                            result === null ? (null as any) : new U256(result)
                        );
                    } catch (e) {
                        reject(
                            Error(
                                `Expected chain_getNonce to return a value of U256, but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Gets number of the latest block.
     * @returns Number of the latest block.
     */
    public getBestBlockNumber(): Promise<number> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getBestBlockNumber", [])
                .then(result => {
                    if (typeof result === "number") {
                        return resolve(result);
                    }
                    reject(
                        Error(
                            `Expected chain_getBestBlockNumber to return a number, but it returned ${result}`
                        )
                    );
                })
                .catch(reject);
        });
    }

    /**
     * Gets block hash of given blockNumber.
     * @param blockNumber The block number of which to get the block hash of.
     * @returns BlockHash, if block exists. Else, returns null.
     */
    public getBlockHash(blockNumber: number): Promise<H256 | null> {
        if (!isNonNegativeInterger(blockNumber)) {
            throw Error(
                `Expected the first argument of getBlockHash to be a non-negative integer but found ${blockNumber}`
            );
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getBlockHash", [blockNumber])
                .then(result => {
                    try {
                        resolve(result === null ? null : new H256(result));
                    } catch (e) {
                        reject(
                            Error(
                                `Expected chain_getBlockHash to return either null or a value of H256, but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Gets block of given block hash.
     * @param hashOrNumber The block hash or number of which to get the block of
     * @returns Block, if block exists. Else, returns null.
     */
    public async getBlock(
        hashOrNumber: H256 | string | number
    ): Promise<Block | null> {
        let result;
        if (hashOrNumber instanceof H256 || typeof hashOrNumber === "string") {
            result = await this.rpc.sendRpcRequest("chain_getBlockByHash", [
                `0x${H256.ensure(hashOrNumber).value}`
            ]);
        } else if (typeof hashOrNumber === "number") {
            result = await this.rpc.sendRpcRequest("chain_getBlockByNumber", [
                hashOrNumber
            ]);
        } else {
            throw Error(
                `Expected the first argument of getBlock to be either a number or an H256 value but found ${hashOrNumber}`
            );
        }
        try {
            return result === null ? null : Block.fromJSON(result);
        } catch (e) {
            throw Error(
                `Expected chain_getBlock to return either null or JSON of Block, but an error occurred: ${e.toString()}`
            );
        }
    }

    /**
     * Gets asset scheme of given hash of AssetMintTransaction.
     * @param txhash The tx hash of AssetMintTransaction.
     * @param shardId The shard id of Asset Scheme.
     * @param worldId The world id of Asset Scheme.
     * @returns AssetScheme, if asset scheme exists. Else, returns null.
     */
    public getAssetSchemeByHash(
        txhash: H256 | string,
        shardId: number,
        worldId: number
    ): Promise<AssetScheme | null> {
        if (!H256.check(txhash)) {
            throw Error(
                `Expected the first arugment of getAssetSchemeByHash to be an H256 value but found ${txhash}`
            );
        }
        if (!isShardIdValue(shardId)) {
            throw Error(
                `Expected the second argument of getAssetSchemeByHash to be a shard ID value but found ${shardId}`
            );
        }
        if (!isWorldIdValue(worldId)) {
            throw Error(
                `Expected the third argument of getAssetSchemeByHash to be a world ID value but found ${shardId}`
            );
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getAssetSchemeByHash", [
                    `0x${H256.ensure(txhash).value}`,
                    shardId,
                    worldId
                ])
                .then(result => {
                    try {
                        resolve(
                            result === null
                                ? null
                                : AssetScheme.fromJSON(result)
                        );
                    } catch (e) {
                        reject(
                            Error(
                                `Expected chain_getAssetSchemeByHash to return either null or JSON of AssetScheme, but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Gets asset scheme of asset type
     * @param assetType The type of Asset.
     * @returns AssetScheme, if asset scheme exists. Else, returns null.
     */
    public getAssetSchemeByType(
        assetType: H256 | string
    ): Promise<AssetScheme | null> {
        if (!H256.check(assetType)) {
            throw Error(
                `Expected the first arugment of getAssetSchemeByType to be an H256 value but found ${assetType}`
            );
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getAssetSchemeByType", [
                    `0x${H256.ensure(assetType).value}`
                ])
                .then(result => {
                    try {
                        resolve(
                            result === null
                                ? null
                                : AssetScheme.fromJSON(result)
                        );
                    } catch (e) {
                        reject(
                            Error(
                                `Expected chain_getAssetSchemeByType to return either null or JSON of AssetScheme, but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Gets asset of given transaction hash and index.
     * @param txhash The tx hash of AssetMintTransaction or AssetTransferTransaction.
     * @param index The index of output in the transaction.
     * @param blockNumber The specific block number to get the asset from
     * @returns Asset, if asset exists, Else, returns null.
     */
    public getAsset(
        txhash: H256 | string,
        index: number,
        blockNumber?: number
    ): Promise<Asset | null> {
        if (!H256.check(txhash)) {
            throw Error(
                `Expected the first argument of getAsset to be an H256 value but found ${txhash}`
            );
        }
        if (!isNonNegativeInterger(index)) {
            throw Error(
                `Expected the second argument of getAsset to be non-negative integer but found ${index}`
            );
        }
        if (blockNumber !== undefined && !isNonNegativeInterger(blockNumber)) {
            throw Error(
                `Expected the third argument of getAsset to be non-negative integer but found ${index}`
            );
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getAsset", [
                    `0x${H256.ensure(txhash).value}`,
                    index,
                    blockNumber
                ])
                .then(result => {
                    if (result === null) {
                        return resolve(null);
                    }
                    try {
                        resolve(
                            Asset.fromJSON({
                                ...result,
                                transactionHash: H256.ensure(txhash).value,
                                transactionOutputIndex: index
                            })
                        );
                    } catch (e) {
                        reject(
                            Error(
                                `Expected chain_getAsset to return either null or JSON of Asset, but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Checks whether an asset is spent or not.
     * @param txhash The tx hash of AssetMintTransaction or AssetTransferTransaction.
     * @param index The index of output in the transaction.
     * @param shardId The shard id of an Asset.
     * @param blockNumber The specific block number to get the asset from.
     * @returns True, if the asset is spent. False, if the asset is not spent. Null, if no such asset exists.
     */
    public isAssetSpent(
        txhash: H256 | string,
        index: number,
        shardId: number,
        blockNumber?: number
    ): Promise<boolean | null> {
        if (!H256.check(txhash)) {
            throw Error(
                `Expected the first argument of isAssetSpent to be an H256 value but found ${txhash}`
            );
        }
        if (!isNonNegativeInterger(index)) {
            throw Error(
                `Expected the second argument of isAssetSpent to be a non-negative integer but found ${index}`
            );
        }
        if (!isShardIdValue(shardId)) {
            throw Error(
                `Expected the third argument of isAssetSpent to be a shard ID value but found ${shardId}`
            );
        }

        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_isAssetSpent", [
                    `0x${H256.ensure(txhash).value}`,
                    index,
                    shardId,
                    blockNumber
                ])
                .then(result => {
                    if (result === null || typeof result === "boolean") {
                        resolve(result);
                    } else {
                        reject(
                            Error(
                                `Expected chain_isAssetSpent to return either null or boolean but it returned ${result}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Gets pending parcels.
     * @returns List of SignedParcel, with each parcel has null for blockNumber/blockHash/parcelIndex.
     */
    public getPendingParcels(): Promise<SignedParcel[]> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getPendingParcels", [])
                .then(result => {
                    if (!Array.isArray(result)) {
                        return reject(
                            Error(
                                `Expected chain_getPendingParcels to return an array but it returned ${result}`
                            )
                        );
                    }
                    try {
                        resolve(result.map(p => SignedParcel.fromJSON(p)));
                    } catch (e) {
                        reject(
                            Error(
                                `Expected chain_getPendingParcels to return an array of JSON of SignedParcel, but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Gets the network ID of the node.
     * @returns A network ID, e.g. "tc".
     */
    public getNetworkId(): Promise<NetworkId> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getNetworkId", [])
                .then(result => {
                    if (typeof result === "string") {
                        resolve(result);
                    } else {
                        reject(
                            Error(
                                `Expected chain_getNetworkId to return a string but it returned ${result}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }
}

function isNonNegativeInterger(value: any): boolean {
    return typeof value === "number" && Number.isInteger(value) && value >= 0;
}

function isShardIdValue(value: any): boolean {
    return (
        typeof value === "number" &&
        Number.isInteger(value) &&
        value >= 0 &&
        value <= 0xffff
    );
}

function isWorldIdValue(value: any): boolean {
    return isShardIdValue(value);
}
