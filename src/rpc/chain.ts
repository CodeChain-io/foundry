import { H160, PlatformAddress } from "codechain-primitives";

import { Rpc } from ".";
import { Asset } from "../core/Asset";
import { AssetScheme } from "../core/AssetScheme";
import { Block } from "../core/Block";
import { H256 } from "../core/H256";
import { H512 } from "../core/H512";
import { Invoice } from "../core/Invoice";
import { SignedTransaction } from "../core/SignedTransaction";
import { Text } from "../core/Text";
import { Transaction } from "../core/Transaction";
import { fromJSONToSignedTransaction } from "../core/transaction/json";
import { NetworkId } from "../core/types";
import { U64 } from "../core/U64";

export class ChainRpc {
    private rpc: Rpc;
    private transactionSigner?: string;
    private transactionFee?: number;

    /**
     * @hidden
     */
    constructor(
        rpc: Rpc,
        options: { transactionSigner?: string; transactionFee?: number }
    ) {
        const { transactionSigner, transactionFee } = options;
        this.rpc = rpc;
        this.transactionSigner = transactionSigner;
        this.transactionFee = transactionFee;
    }

    /**
     * Sends SignedTransaction to CodeChain's network.
     * @param tx SignedTransaction
     * @returns SignedTransaction's hash.
     */
    public sendSignedTransaction(tx: SignedTransaction): Promise<H256> {
        if (!(tx instanceof SignedTransaction)) {
            throw Error(
                `Expected the first argument of sendSignedTransaction to be SignedTransaction but found ${tx}`
            );
        }
        return new Promise((resolve, reject) => {
            const bytes = Array.from(tx.rlpBytes())
                .map(
                    byte =>
                        byte < 0x10
                            ? `0${byte.toString(16)}`
                            : byte.toString(16)
                )
                .join("");
            this.rpc
                .sendRpcRequest("chain_sendSignedTransaction", [`0x${bytes}`])
                .then(result => {
                    try {
                        resolve(new H256(result));
                    } catch (e) {
                        reject(
                            Error(
                                `Expected sendSignedTransaction() to return a value of H256, but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Signs a tx with the given account and sends it to CodeChain's network.
     * @param tx The tx to send
     * @param options.account The account to sign the tx
     * @param options.passphrase The account's passphrase
     * @param options.seq The seq of the tx
     * @param options.fee The fee of the tx
     * @returns SignedTransaction's hash
     * @throws When the given account cannot afford to pay the fee
     * @throws When the given fee is too low
     * @throws When the given seq does not match
     * @throws When the given account is unknown
     * @throws When the given passphrase does not match
     */
    public async sendTransaction(
        tx: Transaction,
        options?: {
            account?: PlatformAddress | string;
            passphrase?: string;
            seq?: number | null;
            fee?: U64 | string | number;
        }
    ): Promise<H256> {
        if (!(tx instanceof Transaction)) {
            throw Error(
                `Expected the first argument of sendTransaction to be a Transaction but found ${tx}`
            );
        }
        const {
            account = this.transactionSigner,
            passphrase,
            fee = this.transactionFee
        } = options || { passphrase: undefined };
        if (!account) {
            throw Error("The account to sign the tx is not specified");
        } else if (!PlatformAddress.check(account)) {
            throw Error(
                `Expected account param of sendTransaction to be a PlatformAddress value but found ${account}`
            );
        }
        const { seq = await this.getSeq(account) } = options || {};
        tx.setSeq(seq!);
        if (!fee) {
            throw Error("The fee of the tx is not specified");
        }
        tx.setFee(fee);
        const address = PlatformAddress.ensure(account);
        const sig = await this.rpc.account.sign(tx.hash(), address, passphrase);
        return this.sendSignedTransaction(new SignedTransaction(tx, sig));
    }

    /**
     * Gets SignedTransaction of given hash. Else returns null.
     * @param hash SignedTransaction's hash
     * @returns SignedTransaction, or null when SignedTransaction was not found.
     */
    public getTransaction(
        hash: H256 | string
    ): Promise<SignedTransaction | null> {
        if (!H256.check(hash)) {
            throw Error(
                `Expected the first argument of getTransaction to be an H256 value but found ${hash}`
            );
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getTransaction", [
                    `0x${H256.ensure(hash).value}`
                ])
                .then(result => {
                    try {
                        resolve(
                            result === null
                                ? null
                                : fromJSONToSignedTransaction(result)
                        );
                    } catch (e) {
                        reject(
                            Error(
                                `Expected chain_getTransaction to return either null or JSON of SignedTransaction, but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Gets invoices of given tx.
     * @param hash The tx hash of which to get the corresponding tx of.
     * @param options.timeout Indicating milliseconds to wait the tx to be confirmed.
     * @returns List of invoice, or null when no such tx exists.
     */
    public async getInvoice(
        hash: H256 | string,
        options: { timeout?: number } = {}
    ): Promise<Invoice | null> {
        if (!H256.check(hash)) {
            throw Error(
                `Expected the first argument of getInvoice to be an H256 value but found ${hash}`
            );
        }
        const attemptToGet = async () => {
            return this.rpc.sendRpcRequest("chain_getInvoice", [
                `0x${H256.ensure(hash).value}`
            ]);
        };
        const { timeout } = options;
        if (
            timeout !== undefined &&
            (typeof timeout !== "number" || timeout < 0)
        ) {
            throw Error(
                `Expected timeout param of getInvoice to be non-negative number but found ${timeout}`
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
            return Invoice.fromJSON(result);
        } catch (e) {
            throw Error(
                `Expected chain_getInvoice to return either null or JSON of Invoice, but an error occurred: ${e.toString()}`
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
     * @param tracker The tracker of which to get the corresponding transaction of.
     * @returns A transaction, or null when transaction of given hash not exists.
     */
    public getTransactionByTracker(
        tracker: H256 | string
    ): Promise<SignedTransaction | null> {
        if (!H256.check(tracker)) {
            throw Error(
                `Expected the first argument of getTransactionByTracker to be an H256 value but found ${tracker}`
            );
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getTransactionByTracker", [
                    `0x${H256.ensure(tracker).value}`
                ])
                .then(result => {
                    try {
                        resolve(
                            result === null
                                ? null
                                : fromJSONToSignedTransaction(result)
                        );
                    } catch (e) {
                        reject(
                            Error(
                                `Expected chain_getTransactionByTracker to return either null or JSON of Transaction, but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Gets invoice of a transaction of given tracker.
     * @param tracker The transaction hash of which to get the corresponding transaction of.
     * @param options.timeout Indicating milliseconds to wait the transaction to be confirmed.
     * @returns Invoice, or null when transaction of given hash not exists.
     */
    public async getInvoicesByTracker(
        tracker: H256 | string,
        options: { timeout?: number } = {}
    ): Promise<Invoice[]> {
        if (!H256.check(tracker)) {
            throw Error(
                `Expected the first argument of getInvoicesByTracker to be an H256 value but found ${tracker}`
            );
        }
        const attemptToGet = async () => {
            return this.rpc.sendRpcRequest("chain_getInvoicesByTracker", [
                `0x${H256.ensure(tracker).value}`
            ]);
        };
        const startTime = Date.now();
        const { timeout } = options;
        if (
            timeout !== undefined &&
            (typeof timeout !== "number" || timeout < 0)
        ) {
            throw Error(
                `Expected timeout param of getInvoicesByTracker to be non-negative number but found ${timeout}`
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
        if (result == null) {
            return [];
        }
        try {
            return result.map(Invoice.fromJSON);
        } catch (e) {
            throw Error(
                `Expected chain_getInvoicesByTracker to return JSON of Invoice[], but an error occurred: ${e.toString()}. received: ${JSON.stringify(
                    result
                )}`
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
    ): Promise<U64> {
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
                        // FIXME: Need to discuss changing the return type to `U64 | null`. It's a
                        // breaking change.
                        resolve(
                            result === null ? (null as any) : new U64(result)
                        );
                    } catch (e) {
                        reject(
                            Error(
                                `Expected chain_getBalance to return a value of U64, but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Gets seq of an account of given address, recorded in the block of given blockNumber. If blockNumber is not given, then returns seq recorded in the most recent block.
     * @param address An account address
     * @param blockNumber The specific block number to get account's seq at given address.
     * @returns Seq of account at the specified block, or null if no such block exists.
     */
    public getSeq(
        address: PlatformAddress | string,
        blockNumber?: number
    ): Promise<number> {
        if (!PlatformAddress.check(address)) {
            throw Error(
                `Expected the first argument of getSeq to be a PlatformAddress value but found ${address}`
            );
        }
        if (blockNumber !== undefined && !isNonNegativeInterger(blockNumber)) {
            throw Error(
                `Expected the second argument of getSeq to be a number but found ${blockNumber}`
            );
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getSeq", [
                    `${PlatformAddress.ensure(address).value}`,
                    blockNumber
                ])
                .then(async result => {
                    if (result == null) {
                        throw Error("chain_getSeq returns undefined");
                    }
                    resolve(result);
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
     * Gets asset scheme of given tracker of the mint transaction.
     * @param tracker The tracker of the mint transaction.
     * @param shardId The shard id of Asset Scheme.
     * @param blockNumber The specific block number to get the asset scheme from
     * @returns AssetScheme, if asset scheme exists. Else, returns null.
     */
    public getAssetSchemeByTracker(
        tracker: H256 | string,
        shardId: number,
        blockNumber?: number | null
    ): Promise<AssetScheme | null> {
        if (!H256.check(tracker)) {
            throw Error(
                `Expected the first arugment of getAssetSchemeByTracker to be an H256 value but found ${tracker}`
            );
        }
        if (!isShardIdValue(shardId)) {
            throw Error(
                `Expected the second argument of getAssetSchemeByTracker to be a shard ID value but found ${shardId}`
            );
        }
        if (blockNumber !== undefined && !isNonNegativeInterger(blockNumber)) {
            throw Error(
                `Expected the third argument of getAssetSchemeByTracker to be non-negative integer but found ${blockNumber}`
            );
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getAssetSchemeByTracker", [
                    `0x${H256.ensure(tracker).value}`,
                    shardId,
                    blockNumber
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
                                `Expected chain_getAssetSchemeByTracker to return either null or JSON of AssetScheme, but an error occurred: ${e.toString()}`
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
     * @param shardId The shard id of Asset Scheme.
     * @param blockNumber The specific block number to get the asset scheme from
     * @returns AssetScheme, if asset scheme exists. Else, returns null.
     */
    public getAssetSchemeByType(
        assetType: H160 | string,
        shardId: number,
        blockNumber?: number | null
    ): Promise<AssetScheme | null> {
        if (!H160.check(assetType)) {
            throw Error(
                `Expected the first arugment of getAssetSchemeByType to be an H160 value but found ${assetType}`
            );
        }
        if (!isShardIdValue(shardId)) {
            throw Error(
                `Expected the second argument of getAssetSchemeByTracker to be a shard ID value but found ${shardId}`
            );
        }
        if (blockNumber !== undefined && !isNonNegativeInterger(blockNumber)) {
            throw Error(
                `Expected the second argument of getAssetSchemeByType to be non-negative integer but found ${blockNumber}`
            );
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getAssetSchemeByType", [
                    `0x${H160.ensure(assetType).value}`,
                    shardId,
                    blockNumber
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
     * @param tracker The tracker of previous input transaction.
     * @param index The index of output in the transaction.
     * @param shardId The shard id of output in the transaction.
     * @param blockNumber The specific block number to get the asset from
     * @returns Asset, if asset exists, Else, returns null.
     */
    public getAsset(
        tracker: H256 | string,
        index: number,
        shardId: number,
        blockNumber?: number
    ): Promise<Asset | null> {
        if (!H256.check(tracker)) {
            throw Error(
                `Expected the first argument of getAsset to be an H256 value but found ${tracker}`
            );
        }
        if (!isNonNegativeInterger(index)) {
            throw Error(
                `Expected the second argument of getAsset to be non-negative integer but found ${index}`
            );
        }
        if (!isShardIdValue(shardId)) {
            throw Error(
                `Expected the second argument of getAssetSchemeByTracker to be a shard ID value but found ${shardId}`
            );
        }
        if (blockNumber !== undefined && !isNonNegativeInterger(blockNumber)) {
            throw Error(
                `Expected the third argument of getAsset to be non-negative integer but found ${blockNumber}`
            );
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getAsset", [
                    `0x${H256.ensure(tracker).value}`,
                    index,
                    shardId,
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
                                shardId,
                                tracker: H256.ensure(tracker).value,
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
     * Gets the text of the given hash of tx with Store type.
     * @param txHash The tx hash of the Store tx.
     * @param blockNumber The specific block number to get the text from
     * @returns Text, if text exists. Else, returns null.
     */
    public getText(
        txHash: H256 | string,
        blockNumber?: number | null
    ): Promise<Text | null> {
        if (!H256.check(txHash)) {
            throw Error(
                `Expected the first arugment of getText to be an H256 value but found ${txHash}`
            );
        }
        if (blockNumber !== undefined && !isNonNegativeInterger(blockNumber)) {
            throw Error(
                `Expected the second argument of getText to be non-negative integer but found ${blockNumber}`
            );
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getText", [
                    `0x${H256.ensure(txHash).value}`,
                    blockNumber
                ])
                .then(result => {
                    try {
                        resolve(result === null ? null : Text.fromJSON(result));
                    } catch (e) {
                        reject(
                            Error(
                                `Expected chain_getText to return either null or JSON of Text, but an error occurred: ${e.toString()}`
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
     * Gets pending transactions.
     * @returns List of SignedTransaction, with each tx has null for blockNumber/blockHash/transactionIndex.
     */
    public getPendingTransactions(): Promise<SignedTransaction[]> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getPendingTransactions", [])
                .then(result => {
                    if (!Array.isArray(result)) {
                        return reject(
                            Error(
                                `Expected chain_getPendingTransactions to return an array but it returned ${result}`
                            )
                        );
                    }
                    try {
                        resolve(result.map(fromJSONToSignedTransaction));
                    } catch (e) {
                        reject(
                            Error(
                                `Expected chain_getPendingTransactions to return an array of JSON of SignedTransaction, but an error occurred: ${e.toString()}`
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
