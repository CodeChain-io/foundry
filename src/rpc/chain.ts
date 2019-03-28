import {
    H160,
    H160Value,
    H256,
    H256Value,
    H512,
    H512Value,
    PlatformAddress,
    PlatformAddressValue,
    U64,
    U64Value
} from "codechain-primitives";

import { Rpc } from ".";
import { Asset } from "../core/Asset";
import { AssetScheme } from "../core/AssetScheme";
import { Block } from "../core/Block";
import { SignedTransaction } from "../core/SignedTransaction";
import { Text } from "../core/Text";
import { Transaction } from "../core/Transaction";
import { fromJSONToSignedTransaction } from "../core/transaction/json";
import { NetworkId } from "../core/types";

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
            const bytes = tx.rlpBytes().toString("hex");
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
            account?: PlatformAddressValue;
            passphrase?: string;
            seq?: number | null;
            fee?: U64Value;
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
        const sig = await this.rpc.account.sign(
            tx.unsignedHash(),
            address,
            passphrase
        );
        return this.sendSignedTransaction(new SignedTransaction(tx, sig));
    }

    /**
     * Gets SignedTransaction of given hash. Else returns null.
     * @param hash SignedTransaction's hash
     * @returns SignedTransaction, or null when SignedTransaction was not found.
     */
    public getTransaction(hash: H256Value): Promise<SignedTransaction | null> {
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
     * Queries whether the chain has the transaction of given tx.
     * @param hash The tx hash of which to get the corresponding tx of.
     * @returns boolean when transaction of given hash not exists.
     */
    public async containTransaction(hash: H256Value): Promise<boolean> {
        if (!H256.check(hash)) {
            throw Error(
                `Expected the first argument of containTransaction to be an H256 value but found ${hash}`
            );
        }
        const result = await this.rpc.sendRpcRequest(
            "chain_containTransaction",
            [`0x${H256.ensure(hash).value}`]
        );
        try {
            return JSON.parse(result);
        } catch (e) {
            throw Error(
                `Expected chain_containTransaction to return JSON of boolean, but an error occurred: ${e.toString()}`
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
        address: PlatformAddressValue,
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
        regularKey: H512Value,
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
     * Gets the shard id of the given hash of a CreateShard transaction.
     * @param hash A transaction hash of a CreateShard transaction.
     * @param blockNumber A block number.
     * @returns A shard id.
     */
    public getShardIdByHash(
        hash: H256Value,
        blockNumber?: number
    ): Promise<number | null> {
        if (!H256.check(hash)) {
            throw Error(
                `Expected the first argument of getShardIdByHash to be an H256 value but found ${hash}`
            );
        }
        if (blockNumber !== undefined && !isNonNegativeInterger(blockNumber)) {
            throw Error(
                `Expected the second argument of getShardIdByHash to be a number but found ${blockNumber}`
            );
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getShardIdByHash", [
                    H256.ensure(hash).toJSON(),
                    blockNumber
                ])
                .then(result => {
                    if (result === null || typeof result === "number") {
                        resolve(result);
                    } else {
                        reject(
                            Error(
                                `Expected chain_getShardIdByHash to return either number or null but it returned ${result}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Gets the owners of the shard.
     * @param shardId A shard id.
     * @returns The platform addresses of the owners.
     */
    public getShardOwners(
        shardId: number,
        blockNumber?: number
    ): Promise<PlatformAddress[] | null> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getShardOwners", [shardId, blockNumber])
                .then(result => {
                    try {
                        resolve(
                            result === null
                                ? null
                                : (result as string[]).map(str =>
                                      PlatformAddress.ensure(str)
                                  )
                        );
                    } catch (e) {
                        reject(
                            Error(
                                `Expected chain_getShardOwners to return either null or an array of PlatformAddress, but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Gets the users of the shard.
     * @param shardId A shard id.
     * @returns The platform addresses of the users.
     */
    public getShardUsers(
        shardId: number,
        blockNumber?: number
    ): Promise<PlatformAddress[] | null> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getShardUsers", [shardId, blockNumber])
                .then(result => {
                    try {
                        resolve(
                            result === null
                                ? null
                                : (result as string[]).map(str =>
                                      PlatformAddress.ensure(str)
                                  )
                        );
                    } catch (e) {
                        reject(
                            Error(
                                `Expected chain_getShardUsers to return either null or an array of PlatformAddress, but an error occurred: ${e.toString()}`
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
        tracker: H256Value
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
     * Gets results of a transaction of given tracker.
     * @param tracker The transaction hash of which to get the corresponding transaction of.
     * @param options.timeout Indicating milliseconds to wait the transaction to be confirmed.
     * @returns List of boolean, or null when transaction of given hash not exists.
     */
    public async getTransactionResultsByTracker(
        tracker: H256Value,
        options: { timeout?: number } = {}
    ): Promise<boolean[]> {
        if (!H256.check(tracker)) {
            throw Error(
                `Expected the first argument of getTransactionResultsByTracker to be an H256 value but found ${tracker}`
            );
        }
        const attemptToGet = async () => {
            return this.rpc.sendRpcRequest(
                "chain_getTransactionResultsByTracker",
                [`0x${H256.ensure(tracker).value}`]
            );
        };
        const startTime = Date.now();
        const { timeout } = options;
        if (
            timeout !== undefined &&
            (typeof timeout !== "number" || timeout < 0)
        ) {
            throw Error(
                `Expected timeout param of getTransactionResultsByTracker to be non-negative number but found ${timeout}`
            );
        }
        let result = await attemptToGet();
        while (
            result == null &&
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
            return result.map(JSON.parse);
        } catch (e) {
            throw Error(
                `Expected chain_getTransactionResultsByTracker to return JSON of boolean[], but an error occurred: ${e.toString()}. received: ${JSON.stringify(
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
        address: PlatformAddressValue,
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
     * Gets a hint to find out why the transaction failed.
     * @param transactionHash A transaction hash from which the error hint would get.
     * @returns Null if the transaction is not involved in the chain or succeeded. If the transaction failed, this should return the reason for the transaction failing.
     */
    public async getErrorHint(
        transactionHash: H256Value
    ): Promise<string | null> {
        if (!H256.check(transactionHash)) {
            throw Error(
                `Expected the first argument of getErrorHint to be an H256 value but found ${transactionHash}`
            );
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getErrorHint", [
                    `0x${H256.ensure(transactionHash).value}`
                ])
                .then(result => {
                    if (typeof result === "string" || result == null) {
                        return resolve(result);
                    }
                    reject(
                        Error(
                            `Expected chain_getErrorHint to return either null or value of string, but it returned ${result}`
                        )
                    );
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
        address: PlatformAddressValue,
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
        hashOrNumber: H256Value | number
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
        tracker: H256Value,
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
        assetType: H160Value,
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
                `Expected the second argument of getAssetSchemeByType to be a shard ID value but found ${shardId}`
            );
        }
        if (blockNumber !== undefined && !isNonNegativeInterger(blockNumber)) {
            throw Error(
                `Expected the third argument of getAssetSchemeByType to be non-negative integer but found ${blockNumber}`
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
        tracker: H256Value,
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
                `Expected the third argument of getAsset to be a shard ID value but found ${shardId}`
            );
        }
        if (blockNumber !== undefined && !isNonNegativeInterger(blockNumber)) {
            throw Error(
                `Expected the forth argument of getAsset to be non-negative integer but found ${blockNumber}`
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
        txHash: H256Value,
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
                        resolve(result == null ? null : Text.fromJSON(result));
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
        txhash: H256Value,
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
                                `Expected chain_isAssetSpent to return either null or a boolean but it returned ${result}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Gets pending transactions that have the insertion timestamp within the given range.
     * @param from The lower bound of the insertion timestamp.
     * @param to The upper bound of the insertion timestamp.
     * @returns List of SignedTransaction, with each tx has null for blockNumber/blockHash/transactionIndex.
     */
    public getPendingTransactions(
        from?: number | null,
        to?: number | null
    ): Promise<{
        transactions: SignedTransaction[];
        lastTimestamp: number | null;
    }> {
        if (from != null && !isNonNegativeInterger(from)) {
            throw Error(
                `Expected the first argument of getPendingTransactions to be a non-negative integer but found ${from}`
            );
        }
        if (to != null && !isNonNegativeInterger(to)) {
            throw Error(
                `Expected the second argument of getPendingTransactions to be a non-negative integer but found ${to}`
            );
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getPendingTransactions", [from, to])
                .then(result => {
                    try {
                        const resultTransactions = result.transactions;
                        const resultLastTimestamp = result.lastTimestamp;
                        if (!Array.isArray(resultTransactions)) {
                            return reject(
                                Error(
                                    `Expected chain_getPendingTransactions to return an object whose property "transactions" is of array type but it is ${resultTransactions}`
                                )
                            );
                        }
                        if (
                            resultLastTimestamp !== null &&
                            typeof resultLastTimestamp !== "number"
                        ) {
                            return reject(
                                Error(
                                    `Expected chain_getPendingTransactions to return an object containing a number but it returned ${resultLastTimestamp}`
                                )
                            );
                        }
                        resolve({
                            transactions: resultTransactions.map(
                                fromJSONToSignedTransaction
                            ),
                            lastTimestamp: resultLastTimestamp
                        });
                    } catch (e) {
                        reject(
                            Error(
                                `Expected chain_getPendingTransactions to return an object who has transactions and lastTimestamp properties, but an error occurred: ${e.toString()}`
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

    /**
     * Gets the number of shards, at the state of the given blockNumber
     * @param blockNumber A block number.
     * @returns A number of shards
     */
    public getNumberOfShards(blockNumber?: number): Promise<number> {
        if (blockNumber !== undefined && !isNonNegativeInterger(blockNumber)) {
            throw Error(
                `Expected the first argument of getNumberOfShards to be a number but found ${blockNumber}`
            );
        }

        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getNumberOfShards", [blockNumber])
                .then(result => {
                    if (result === null || typeof result === "number") {
                        resolve(result);
                    } else {
                        reject(
                            Error(
                                `Expected chain_getNumberOfShards to return a number, but it returned ${result}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Gets the platform account in the genesis block
     * @returns The platform addresses in the genesis block.
     */
    public getGenesisAccounts(): Promise<PlatformAddress[]> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getGenesisAccounts", [])
                .then(result => {
                    try {
                        resolve(
                            (result as string[]).map(str =>
                                PlatformAddress.ensure(str)
                            )
                        );
                    } catch (e) {
                        reject(
                            Error(
                                `Expected chain_getGenesisAccounts to return an array of PlatformAddress, but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Gets the root of the shard, at the state of the given blockNumber.
     * @param shardId A shard Id.
     * @param blockNumber A block number.
     * @returns The hash of root of the shard.
     */
    public getShardRoot(
        shardId: number,
        blockNumber?: number
    ): Promise<H256 | null> {
        if (!isShardIdValue(shardId)) {
            throw Error(
                `Expected the first argument of getShardRoot to be a shard ID value but found ${shardId}`
            );
        }
        if (blockNumber !== undefined && !isNonNegativeInterger(blockNumber)) {
            throw Error(
                `Expected the second argument of getShardRoot to be a non-negative integer but found ${blockNumber}`
            );
        }

        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getShardRoot", [shardId, blockNumber])
                .then(result => {
                    try {
                        resolve(result === null ? null : new H256(result));
                    } catch (e) {
                        reject(
                            Error(
                                `Expected chain_getShardRoot to return either null or a value of H256, but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Gets the mining reward of the given block number.
     * @param blockNumber A block nubmer.
     * @returns The amount of mining reward, or null if the given block number is not mined yet.
     */
    public getMiningReward(blockNumber: number): Promise<U64 | null> {
        if (!isNonNegativeInterger(blockNumber)) {
            throw Error(
                `Expected the argument of getMiningReward to be a non-negative integer but found ${blockNumber}`
            );
        }

        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getMiningReward", [blockNumber])
                .then(result => {
                    try {
                        resolve(result === null ? null : new U64(result));
                    } catch (e) {
                        reject(
                            Error(
                                `Expected chain_getMiningReward to return either null or a value of U64, but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Executes the transactions.
     * @param tx A transaction to execute.
     * @param sender A platform address of sender.
     * @returns True, if the transaction is executed successfully. False, if the transaction is not executed.
     */
    public executeTransaction(
        tx: Transaction,
        sender: PlatformAddressValue
    ): Promise<string | null> {
        if (!(tx instanceof Transaction)) {
            throw Error(
                `Expected the first argument of executeTransaction to be a Transaction but found ${tx}`
            );
        }
        if (!PlatformAddress.check(sender)) {
            throw Error(
                `Expected the second argument of executeTransaction to be a PlatformAddress value but found ${PlatformAddress}`
            );
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_executeTransaction", [
                    tx.toJSON(),
                    PlatformAddress.ensure(sender).toString()
                ])
                .then(resolve)
                .catch(reject);
        });
    }

    /**
     * Gets the id of the latest block.
     * @returns A number and the hash of the latest block.
     */
    public getBestBlockId(): Promise<{ hash: H256; number: number }> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getBestBlockId", [])
                .then(result => {
                    if (
                        result.hasOwnProperty("hash") &&
                        result.hasOwnProperty("number")
                    ) {
                        return resolve(result);
                    } else {
                        reject(
                            Error(
                                `Expected chain_getBestBlockId to return a number and an H256 value , but it returned ${result}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Gets the number of transactions within a block that corresponds with the given hash.
     * @param hash The block hash.
     * @returns A number of transactions within a block.
     */
    public getBlockTransactionCountByHash(
        hash: H256Value
    ): Promise<number | null> {
        if (!H256.check(hash)) {
            throw Error(
                `Expected the first argument of getBlockTransactionCountByHash to be an H256 value but found ${hash}`
            );
        }

        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getBlockTransactionCountByHash", [hash])
                .then(result => {
                    if (result == null || typeof result === "number") {
                        return resolve(result);
                    } else {
                        reject(
                            Error(
                                `Expected chain_getBlockTransactionCountByHash to return either null or a number but it returned ${result}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Gets the count of the pending transactions within the given range from the transaction queues.
     * @param from The lower bound of collected pending transactions. If null, there is no lower bound.
     * @param to The upper bound of collected pending transactions. If null, there is no upper bound.
     * @returns The count of the pending transactions.
     */
    public getPendingTransactionsCount(
        from?: number | null,
        to?: number | null
    ): Promise<number> {
        if (from != null && !isNonNegativeInterger(from)) {
            throw Error(
                `Expected the first argument of getPendingTransactions to be a non-negative integer but found ${from}`
            );
        }
        if (to != null && !isNonNegativeInterger(to)) {
            throw Error(
                `Expected the second argument of getPendingTransactions to be a non-negative integer but found ${to}`
            );
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_getPendingTransactionsCount", [from, to])
                .then(result => {
                    if (typeof result === "number") {
                        resolve(result);
                    } else {
                        reject(
                            Error(
                                `Expected chain_getPendingTransactionsCount to return a number but returned ${result}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Execute the inputs of the AssetTransfer transaction in the CodeChain VM.
     * @param transaction The transaction that its inputs will be executed.
     * @param parameters Parameters of the outputs as an array.
     * @param indices Indices of inputs to run in VM.
     * @returns The results of VM execution.
     */
    public executeVM(
        transaction: SignedTransaction,
        parameters: string[][],
        indices: number[]
    ): Promise<string[]> {
        if (!(transaction instanceof SignedTransaction)) {
            throw Error(
                `Expected the first argument of executeVM to be a Transaction but found ${transaction}`
            );
        }
        if (parameters.length !== indices.length) {
            throw Error(`The length of paramters and indices must be equal`);
        }
        const params = parameters.map(parameter =>
            parameter.map(string => {
                if (/^[0-9a-f]+$/g.test(string)) {
                    return [...Buffer.from(string, "hex")];
                } else {
                    throw Error(
                        `Parameters should be array of array of hex string`
                    );
                }
            })
        );
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("chain_executeVM", [
                    transaction,
                    params,
                    indices
                ])
                .then(result => {
                    if (result.every((str: any) => typeof str === "string")) {
                        resolve(result);
                    } else {
                        throw Error(`Failed to execute VM`);
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
