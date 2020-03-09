import {
    AssetAddress,
    AssetAddressValue,
    H128,
    H160,
    H160Value,
    H256,
    H256Value,
    H512,
    H512Value,
    PlatformAddress,
    PlatformAddressValue,
    U256,
    U64,
    U64Value
} from "foundry-primitives";

import { Asset } from "./Asset";
import { AssetScheme } from "./AssetScheme";
import { Block } from "./Block";
import { Script } from "./Script";
import { SignedTransaction } from "./SignedTransaction";
import { Transaction } from "./Transaction";
import { AssetMintOutput } from "./transaction/AssetMintOutput";
import { AssetOutPoint } from "./transaction/AssetOutPoint";
import { AssetTransferInput, Timelock } from "./transaction/AssetTransferInput";
import { AssetTransferOutput } from "./transaction/AssetTransferOutput";
import { ChangeAssetScheme } from "./transaction/ChangeAssetScheme";
import { CreateShard } from "./transaction/CreateShard";
import { Custom } from "./transaction/Custom";
import { IncreaseAssetSupply } from "./transaction/IncreaseAssetSupply";
import { MintAsset } from "./transaction/MintAsset";
import { Order } from "./transaction/Order";
import { OrderOnTransfer } from "./transaction/OrderOnTransfer";
import { Pay } from "./transaction/Pay";
import { Remove } from "./transaction/Remove";
import { SetRegularKey } from "./transaction/SetRegularKey";
import { SetShardOwners } from "./transaction/SetShardOwners";
import { SetShardUsers } from "./transaction/SetShardUsers";
import { Store } from "./transaction/Store";
import { TransferAsset } from "./transaction/TransferAsset";
import { UnwrapCCC } from "./transaction/UnwrapCCC";
import { WrapCCC } from "./transaction/WrapCCC";
import { NetworkId } from "./types";

export class Core {
    public static classes = {
        // Data
        H128,
        H160,
        H256,
        H512,
        U256,
        U64,
        // Block
        Block,
        // Transaction
        Transaction,
        SignedTransaction,
        // Transaction
        Pay,
        SetRegularKey,
        CreateShard,
        SetShardOwners,
        SetShardUsers,
        WrapCCC,
        Store,
        Remove,
        Custom,
        AssetTransferInput,
        AssetTransferOutput,
        AssetOutPoint,
        // Asset and AssetScheme
        Asset,
        AssetScheme,
        // Script
        Script,
        // Addresses
        PlatformAddress,
        AssetAddress
    };

    public classes = Core.classes;
    private networkId: NetworkId;

    /**
     * @param params.networkId The network id of CodeChain.
     */
    constructor(params: { networkId: NetworkId }) {
        const { networkId } = params;
        this.networkId = networkId;
    }

    /**
     * Creates Pay type which pays the value quantity of CCC(CodeChain Coin)
     * from the tx signer to the recipient. Who is signing the tx will pay.
     * @param params.recipient The platform account who receives CCC
     * @param params.quantity quantity of CCC to pay
     * @throws Given string for recipient is invalid for converting it to PlatformAddress
     * @throws Given number or string for quantity is invalid for converting it to U64
     */
    public createPayTransaction(params: {
        recipient: PlatformAddressValue;
        quantity: U64Value;
    }): Pay {
        const { recipient, quantity } = params;
        checkPlatformAddressRecipient(recipient);
        checkAmount(quantity);
        return new Pay(
            PlatformAddress.ensure(recipient),
            U64.ensure(quantity),
            this.networkId
        );
    }

    /**
     * Creates SetRegularKey type which sets the regular key of the tx signer.
     * @param params.key The public key of a regular key
     * @throws Given string for key is invalid for converting it to H512
     */
    public createSetRegularKeyTransaction(params: {
        key: H512Value;
    }): SetRegularKey {
        const { key } = params;
        checkKey(key);
        return new SetRegularKey(H512.ensure(key), this.networkId);
    }

    /**
     * Creates CreateShard type which can create new shard
     */
    public createCreateShardTransaction(params: {
        users: Array<PlatformAddressValue>;
    }): CreateShard {
        const { users } = params;
        return new CreateShard(
            {
                users: users.map(PlatformAddress.ensure)
            },
            this.networkId
        );
    }

    public createSetShardOwnersTransaction(params: {
        shardId: number;
        owners: Array<PlatformAddressValue>;
    }): SetShardOwners {
        const { shardId, owners } = params;
        checkShardId(shardId);
        checkOwners(owners);
        return new SetShardOwners(
            {
                shardId,
                owners: owners.map(PlatformAddress.ensure)
            },
            this.networkId
        );
    }

    /**
     * Create SetShardUser type which can change shard users
     * @param params.shardId
     * @param params.users
     */
    public createSetShardUsersTransaction(params: {
        shardId: number;
        users: Array<PlatformAddressValue>;
    }): SetShardUsers {
        const { shardId, users } = params;
        checkShardId(shardId);
        checkUsers(users);
        return new SetShardUsers(
            {
                shardId,
                users: users.map(PlatformAddress.ensure)
            },
            this.networkId
        );
    }

    /**
     * Creates Wrap CCC type which wraps the value quantity of CCC(CodeChain Coin)
     * in a wrapped CCC asset. Who is signing the tx will pay.
     * @param params.shardId A shard ID of the wrapped CCC asset.
     * @param params.lockScriptHash A lock script hash of the wrapped CCC asset.
     * @param params.parameters Parameters of the wrapped CCC asset.
     * @param params.quantity quantity of CCC to pay
     * @throws Given string for a lock script hash is invalid for converting it to H160
     * @throws Given number or string for quantity is invalid for converting it to U64
     */
    public createWrapCCCTransaction(
        params:
            | {
                  shardId: number;
                  lockScriptHash: H160Value;
                  parameters: Buffer[];
                  quantity: U64Value;
                  payer: PlatformAddressValue;
              }
            | {
                  shardId: number;
                  recipient: AssetAddressValue;
                  quantity: U64Value;
                  payer: PlatformAddressValue;
              }
    ): WrapCCC {
        const { shardId, quantity, payer } = params;
        checkShardId(shardId);
        checkAmount(quantity);
        checkPayer(payer);
        let data;
        if ("recipient" in params) {
            checkAssetAddressRecipient(params.recipient);
            data = {
                shardId,
                recipient: AssetAddress.ensure(params.recipient),
                quantity: U64.ensure(quantity),
                payer: PlatformAddress.ensure(payer)
            };
        } else {
            const { lockScriptHash, parameters } = params;
            checkLockScriptHash(lockScriptHash);
            checkParameters(parameters);
            data = {
                shardId,
                lockScriptHash: H160.ensure(lockScriptHash),
                parameters,
                quantity: U64.ensure(quantity),
                payer: PlatformAddress.ensure(payer)
            };
        }
        return new WrapCCC(data, this.networkId);
    }

    /**
     * Creates Store type which store content with certifier on chain.
     * @param params.content Content to store
     * @param params.secret Secret key to sign
     * @param params.certifier Certifier of the text, which is PlatformAddress
     * @param params.signature Signature on the content by the certifier
     * @throws Given string for secret is invalid for converting it to H256
     */
    public createStoreTransaction(
        params:
            | {
                  content: string;
                  certifier: PlatformAddressValue;
                  signature: string;
              }
            | {
                  content: string;
                  secret: H256Value;
              }
    ): Store {
        let storeParams;
        if ("secret" in params) {
            const { content, secret } = params;
            checkSecret(secret);
            storeParams = {
                content,
                secret: H256.ensure(secret)
            };
        } else {
            const { content, certifier, signature } = params;
            checkCertifier(certifier);
            checkSignature(signature);
            storeParams = {
                content,
                certifier: PlatformAddress.ensure(certifier),
                signature
            };
        }
        return new Store(storeParams, this.networkId);
    }

    /**
     * Creates Remove type which remove the text from the chain.
     * @param params.hash Transaction hash which stored the text
     * @param params.secret Secret key to sign
     * @param params.signature Signature on tx hash by the certifier of the text
     * @throws Given string for hash or secret is invalid for converting it to H256
     */
    public createRemoveTransaction(
        params:
            | {
                  hash: H256Value;
                  secret: H256Value;
              }
            | {
                  hash: H256Value;
                  signature: string;
              }
    ): Remove {
        let removeParam = null;
        if ("secret" in params) {
            const { hash, secret } = params;
            checkTransactionHash(hash);
            checkSecret(secret);
            removeParam = {
                hash: H256.ensure(hash),
                secret: H256.ensure(secret)
            };
        } else {
            const { hash, signature } = params;
            checkTransactionHash(hash);
            checkSignature(signature);
            removeParam = {
                hash: H256.ensure(hash),
                signature
            };
        }
        return new Remove(removeParam, this.networkId);
    }

    /**
     * Creates Custom type that will be handled by a specified type handler
     * @param params.handlerId An Id of an type handler which will handle a custom transaction
     * @param params.bytes A custom transaction body
     * @throws Given number for handlerId is invalid for converting it to U64
     */
    public createCustomTransaction(params: {
        handlerId: number;
        bytes: Buffer;
    }): Custom {
        const { handlerId, bytes } = params;
        checkHandlerId(handlerId);
        checkBytes(bytes);
        const customParam = {
            handlerId: U64.ensure(handlerId),
            bytes
        };
        return new Custom(customParam, this.networkId);
    }

    /**
     * Creates asset's scheme.
     * @param params.metadata Any string that describing the asset. For example,
     * stringified JSON containing properties.
     * @param params.supply Total supply of this asset
     * @param params.approver Platform account or null. If account is present, the
     * tx that includes AssetTransferTransaction of this asset must be signed by
     * the approver account.
     * @param params.registrar Platform account or null. The registrar
     * can transfer the asset without unlocking.
     * @throws Given string for approver is invalid for converting it to paltform account
     * @throws Given string for registrar is invalid for converting it to paltform account
     */
    public createAssetScheme(params: {
        shardId: number;
        metadata: string | object;
        supply: U64Value;
        approver?: PlatformAddressValue;
        registrar?: PlatformAddressValue;
        allowedScriptHashes?: H160[];
        pool?: { assetType: H160Value; quantity: number }[];
    }): AssetScheme {
        const {
            shardId,
            supply,
            approver = null,
            registrar = null,
            allowedScriptHashes = null,
            pool = []
        } = params;
        checkMetadata(params.metadata);
        const metadata =
            typeof params.metadata === "string"
                ? params.metadata
                : JSON.stringify(params.metadata);
        checkShardId(shardId);
        checkAmount(supply);
        checkApprover(approver);
        checkregistrar(registrar);
        return new AssetScheme({
            networkId: this.networkId,
            shardId,
            metadata,
            supply: U64.ensure(supply),
            approver:
                approver == null ? null : PlatformAddress.ensure(approver),
            registrar:
                registrar == null ? null : PlatformAddress.ensure(registrar),
            allowedScriptHashes:
                allowedScriptHashes == null ? [] : allowedScriptHashes,
            pool: pool.map(({ assetType, quantity: assetQuantity }) => ({
                assetType: H160.ensure(assetType),
                quantity: U64.ensure(assetQuantity)
            }))
        });
    }

    public createOrder(
        params: {
            assetTypeFrom: H160Value;
            assetTypeTo: H160Value;
            assetTypeFee?: H160Value;
            shardIdFrom: number;
            shardIdTo: number;
            shardIdFee?: number;
            assetQuantityFrom: U64Value;
            assetQuantityTo: U64Value;
            assetQuantityFee?: U64Value;
            originOutputs:
                | AssetOutPoint[]
                | {
                      tracker: H256Value;
                      index: number;
                      assetType: H160Value;
                      shardId: number;
                      quantity: U64Value;
                      lockScriptHash?: H256Value;
                      parameters?: Buffer[];
                  }[];
            expiration: U64Value;
        } & (
            | {
                  lockScriptHashFrom: H160Value;
                  parametersFrom: Buffer[];
              }
            | {
                  recipientFrom: AssetAddressValue;
              }
        ) &
            (
                | {
                      lockScriptHashFee: H160Value;
                      parametersFee: Buffer[];
                  }
                | {
                      recipientFee: AssetAddressValue;
                  }
                | {}
            )
    ): Order {
        const {
            assetTypeFrom,
            assetTypeTo,
            assetTypeFee = H160.zero(),
            shardIdFrom,
            shardIdTo,
            shardIdFee = 0,
            assetQuantityFrom,
            assetQuantityTo,
            assetQuantityFee = 0,
            originOutputs,
            expiration
        } = params;
        checkAssetType(assetTypeFrom);
        checkAssetType(assetTypeTo);
        checkAssetType(assetTypeFee);
        checkShardId(shardIdFrom);
        checkShardId(shardIdTo);
        checkShardId(shardIdFee);
        checkAmount(assetQuantityFrom);
        checkAmount(assetQuantityTo);
        checkAmount(assetQuantityFee);
        checkExpiration(expiration);
        const originOutputsConv: AssetOutPoint[] = [];
        for (let i = 0; i < originOutputs.length; i++) {
            const originOutput = originOutputs[i];
            const {
                tracker,
                index,
                assetType,
                shardId,
                quantity,
                lockScriptHash,
                parameters
            } = originOutput;
            checkAssetOutPoint(originOutput);
            originOutputsConv[i] =
                originOutput instanceof AssetOutPoint
                    ? originOutput
                    : new AssetOutPoint({
                          tracker: H256.ensure(tracker),
                          index,
                          assetType: H160.ensure(assetType),
                          shardId,
                          quantity: U64.ensure(quantity),
                          lockScriptHash: lockScriptHash
                              ? H160.ensure(lockScriptHash)
                              : undefined,
                          parameters
                      });
        }

        const baseParams = {
            assetTypeFrom: H160.ensure(assetTypeFrom),
            assetTypeTo: H160.ensure(assetTypeTo),
            assetTypeFee: H160.ensure(assetTypeFee),
            shardIdFrom,
            shardIdTo,
            shardIdFee,
            assetQuantityFrom: U64.ensure(assetQuantityFrom),
            assetQuantityTo: U64.ensure(assetQuantityTo),
            assetQuantityFee: U64.ensure(assetQuantityFee),
            expiration: U64.ensure(expiration),
            originOutputs: originOutputsConv
        };
        let toParams;
        let feeParams;

        if ("recipientFrom" in params) {
            checkAssetAddressRecipient(params.recipientFrom);
            toParams = {
                recipientFrom: AssetAddress.ensure(params.recipientFrom)
            };
        } else {
            const { lockScriptHashFrom, parametersFrom } = params;
            checkLockScriptHash(lockScriptHashFrom);
            checkParameters(parametersFrom);
            toParams = {
                lockScriptHashFrom: H160.ensure(lockScriptHashFrom),
                parametersFrom
            };
        }

        if ("recipientFee" in params) {
            checkAssetAddressRecipient(params.recipientFee);
            feeParams = {
                recipientFee: AssetAddress.ensure(params.recipientFee)
            };
        } else if ("lockScriptHashFee" in params) {
            const { lockScriptHashFee, parametersFee } = params;
            checkLockScriptHash(lockScriptHashFee);
            checkParameters(parametersFee);
            feeParams = {
                lockScriptHashFee: H160.ensure(lockScriptHashFee),
                parametersFee
            };
        } else {
            feeParams = {
                lockScriptHashFee: H160.ensure("0".repeat(40)),
                parametersFee: []
            };
        }

        return new Order({
            ...baseParams,
            ...toParams,
            ...feeParams
        });
    }
    public createOrderOnTransfer(params: {
        order: Order;
        spentQuantity: U64Value;
        inputFromIndices: number[];
        inputFeeIndices: number[];
        outputFromIndices: number[];
        outputToIndices: number[];
        outputOwnedFeeIndices: number[];
        outputTransferredFeeIndices: number[];
    }) {
        const {
            order,
            spentQuantity,
            inputFromIndices,
            inputFeeIndices,
            outputFromIndices,
            outputToIndices,
            outputOwnedFeeIndices,
            outputTransferredFeeIndices
        } = params;
        checkOrder(order);
        checkAmount(spentQuantity);
        checkIndices(inputFromIndices);
        checkIndices(inputFeeIndices);
        checkIndices(outputFromIndices);
        checkIndices(outputToIndices);
        checkIndices(outputOwnedFeeIndices);
        checkIndices(outputTransferredFeeIndices);

        return new OrderOnTransfer({
            order,
            spentQuantity: U64.ensure(spentQuantity),
            inputFromIndices,
            inputFeeIndices,
            outputFromIndices,
            outputToIndices,
            outputOwnedFeeIndices,
            outputTransferredFeeIndices
        });
    }

    public createMintAssetTransaction(params: {
        scheme:
            | AssetScheme
            | {
                  networkId?: NetworkId;
                  shardId: number;
                  metadata: string | object;
                  approver?: PlatformAddressValue;
                  registrar?: PlatformAddressValue;
                  allowedScriptHashes?: H160[];
                  supply?: U64Value;
              };
        recipient: AssetAddressValue;
        approvals?: string[];
    }): MintAsset {
        const { scheme, recipient, approvals = [] } = params;
        if (scheme != null && typeof scheme !== "object") {
            throw Error(
                `Expected scheme param to be either an AssetScheme or an object but found ${scheme}`
            );
        }
        const {
            networkId = this.networkId,
            shardId,
            approver: approver = null,
            registrar: registrar = null,
            allowedScriptHashes = null,
            supply = U64.MAX_VALUE
        } = scheme;
        checkMetadata(scheme.metadata);
        const metadata =
            typeof scheme.metadata === "string"
                ? scheme.metadata
                : JSON.stringify(scheme.metadata);
        checkAssetAddressRecipient(recipient);
        checkNetworkId(networkId);
        if (shardId === undefined) {
            throw Error(`shardId is undefined`);
        }
        checkShardId(shardId);
        checkApprover(approver);
        checkregistrar(registrar);
        checkAmount(supply);
        return new MintAsset({
            networkId,
            shardId,
            approver:
                approver == null ? null : PlatformAddress.ensure(approver),
            registrar:
                registrar == null ? null : PlatformAddress.ensure(registrar),
            allowedScriptHashes:
                allowedScriptHashes == null ? [] : allowedScriptHashes,
            metadata,
            output: new AssetMintOutput({
                supply: U64.ensure(supply),
                recipient: AssetAddress.ensure(recipient)
            }),
            approvals
        });
    }

    public createChangeAssetSchemeTransaction(params: {
        shardId: number;
        assetType: H160Value;
        seq?: number;
        scheme:
            | AssetScheme
            | {
                  networkId?: NetworkId;
                  metadata: string | object;
                  approver?: PlatformAddressValue;
                  registrar?: PlatformAddressValue;
                  allowedScriptHashes?: H160[];
              };
        approvals?: string[];
    }): ChangeAssetScheme {
        const { shardId, assetType, seq = 0, scheme, approvals = [] } = params;
        if (scheme != null && typeof scheme !== "object") {
            throw Error(
                `Expected scheme param to be either an AssetScheme or an object but found ${scheme}`
            );
        }
        const {
            networkId = this.networkId,
            approver: approver = null,
            registrar: registrar = null,
            allowedScriptHashes = null
        } = scheme;
        checkMetadata(scheme.metadata);
        const metadata =
            typeof scheme.metadata === "string"
                ? scheme.metadata
                : JSON.stringify(scheme.metadata);
        checkNetworkId(networkId);
        checkAssetType(assetType);
        checkApprover(approver);
        checkregistrar(registrar);
        return new ChangeAssetScheme({
            networkId,
            shardId,
            assetType: H160.ensure(assetType),
            seq,
            metadata,
            approver:
                approver == null ? null : PlatformAddress.ensure(approver),
            registrar:
                registrar == null ? null : PlatformAddress.ensure(registrar),
            allowedScriptHashes:
                allowedScriptHashes == null ? [] : allowedScriptHashes,
            approvals
        });
    }

    public createIncreaseAssetSupplyTransaction(params: {
        shardId: number;
        assetType: H160Value;
        seq?: number;
        recipient: AssetAddressValue;
        supply?: U64Value;
        approvals?: string[];
    }): IncreaseAssetSupply {
        const {
            shardId,
            assetType,
            recipient,
            seq = 0,
            supply = U64.MAX_VALUE,
            approvals = []
        } = params;
        checkNetworkId(this.networkId);
        checkShardId(shardId);
        checkAssetType(assetType);
        checkAmount(supply);
        return new IncreaseAssetSupply({
            networkId: this.networkId,
            shardId,
            assetType: H160.ensure(assetType),
            seq,
            output: new AssetMintOutput({
                supply: U64.ensure(supply),
                recipient: AssetAddress.ensure(recipient)
            }),
            approvals
        });
    }

    public createTransferAssetTransaction(params?: {
        burns?: AssetTransferInput[];
        inputs?: AssetTransferInput[];
        outputs?: AssetTransferOutput[];
        orders?: OrderOnTransfer[];
        networkId?: NetworkId;
        metadata?: string | object;
        approvals?: string[];
        expiration?: number;
    }): TransferAsset {
        const {
            burns = [],
            inputs = [],
            outputs = [],
            orders = [],
            networkId = this.networkId,
            metadata = "",
            approvals = [],
            expiration = null
        } = params || {};
        checkMetadata(metadata);
        checkTransferBurns(burns);
        checkTransferInputs(inputs);
        checkTransferOutputs(outputs);
        checkNetworkId(networkId);
        return new TransferAsset({
            burns,
            inputs,
            outputs,
            orders,
            networkId,
            metadata:
                typeof metadata === "string"
                    ? metadata
                    : JSON.stringify(metadata),
            approvals,
            expiration
        });
    }

    public createUnwrapCCCTransaction(params: {
        burn: AssetTransferInput | Asset;
        receiver: PlatformAddressValue;
        networkId?: NetworkId;
    }): UnwrapCCC {
        const { burn, networkId = this.networkId } = params;
        const receiver = PlatformAddress.ensure(params.receiver);
        checkNetworkId(networkId);
        if (burn instanceof Asset) {
            const burnInput = burn.createTransferInput();
            checkTransferBurns([burnInput]);
            return new UnwrapCCC({
                burn: burnInput,
                networkId,
                receiver
            });
        } else {
            checkTransferBurns([burn]);
            return new UnwrapCCC({
                burn,
                networkId,
                receiver
            });
        }
    }

    public createAssetTransferInput(params: {
        assetOutPoint:
            | AssetOutPoint
            | {
                  tracker: H256Value;
                  index: number;
                  assetType: H160Value;
                  shardId: number;
                  quantity: U64Value;
                  lockScriptHash?: H256Value;
                  parameters?: Buffer[];
              };
        timelock?: null | Timelock;
        lockScript?: Buffer;
        unlockScript?: Buffer;
    }): AssetTransferInput {
        const {
            assetOutPoint,
            timelock = null,
            lockScript,
            unlockScript
        } = params;
        checkAssetOutPoint(assetOutPoint);
        checkTimelock(timelock);
        if (lockScript) {
            checkLockScript(lockScript);
        }
        if (unlockScript) {
            checkUnlockScript(unlockScript);
        }
        const {
            tracker,
            index,
            assetType,
            shardId,
            quantity,
            lockScriptHash,
            parameters
        } = assetOutPoint;
        return new AssetTransferInput({
            prevOut:
                assetOutPoint instanceof AssetOutPoint
                    ? assetOutPoint
                    : new AssetOutPoint({
                          tracker: H256.ensure(tracker),
                          index,
                          assetType: H160.ensure(assetType),
                          shardId,
                          quantity: U64.ensure(quantity),
                          lockScriptHash: lockScriptHash
                              ? H160.ensure(lockScriptHash)
                              : undefined,
                          parameters
                      }),
            timelock,
            lockScript,
            unlockScript
        });
    }

    public createAssetOutPoint(params: {
        tracker: H256Value;
        index: number;
        assetType: H160Value;
        shardId: number;
        quantity: U64Value;
    }): AssetOutPoint {
        const { tracker, index, assetType, shardId, quantity } = params;
        checkTracker(tracker);
        checkIndex(index);
        checkAssetType(assetType);
        checkShardId(shardId);
        checkAmount(quantity);
        return new AssetOutPoint({
            tracker: H256.ensure(tracker),
            index,
            assetType: H160.ensure(assetType),
            shardId,
            quantity: U64.ensure(quantity)
        });
    }

    public createAssetTransferOutput(
        params: {
            assetType: H160Value;
            shardId: number;
            quantity: U64Value;
        } & (
            | {
                  recipient: AssetAddressValue;
              }
            | {
                  lockScriptHash: H256Value;
                  parameters: Buffer[];
              }
        )
    ): AssetTransferOutput {
        const { assetType, shardId } = params;
        const quantity = U64.ensure(params.quantity);
        checkAssetType(assetType);
        checkShardId(shardId);
        checkAmount(quantity);
        if ("recipient" in params) {
            const { recipient } = params;
            checkAssetAddressRecipient(recipient);
            return new AssetTransferOutput({
                recipient: AssetAddress.ensure(recipient),
                assetType: H160.ensure(assetType),
                shardId,
                quantity
            });
        } else if ("lockScriptHash" in params && "parameters" in params) {
            const { lockScriptHash, parameters } = params;
            checkLockScriptHash(lockScriptHash);
            checkParameters(parameters);
            return new AssetTransferOutput({
                lockScriptHash: H160.ensure(lockScriptHash),
                parameters,
                assetType: H160.ensure(assetType),
                shardId,
                quantity
            });
        } else {
            throw Error(`Unexpected params: ${params}`);
        }
    }
}

function checkNetworkId(networkId: NetworkId) {
    if (typeof networkId !== "string" || networkId.length !== 2) {
        throw Error(
            `Expected networkId param to be a string of length 2 but found ${networkId}`
        );
    }
}

function checkPlatformAddressRecipient(recipient: PlatformAddressValue) {
    if (!PlatformAddress.check(recipient)) {
        throw Error(
            `Expected recipient param to be a PlatformAddress but found ${recipient}`
        );
    }
}

function checkAssetAddressRecipient(recipient: AssetAddressValue) {
    if (!AssetAddress.check(recipient)) {
        throw Error(
            `Expected recipient param to be a AssetAddress but found ${recipient}`
        );
    }
}

function checkAmount(amount: U64Value) {
    if (!U64.check(amount)) {
        throw Error(
            `Expected amount param to be a U64 value but found ${amount}`
        );
    }
}

function checkExpiration(expiration: U64Value) {
    if (!U64.check(expiration)) {
        throw Error(
            `Expected expiration param to be a U64 value but found ${expiration}`
        );
    }
}

function checkKey(key: H512Value) {
    if (!H512.check(key)) {
        throw Error(`Expected key param to be an H512 value but found ${key}`);
    }
}

function checkShardId(shardId: number) {
    if (
        typeof shardId !== "number" ||
        !Number.isInteger(shardId) ||
        shardId < 0 ||
        shardId > 0xffff
    ) {
        throw Error(
            `Expected shardId param to be a number but found ${shardId}`
        );
    }
}

function checkMetadata(metadata: string | object) {
    if (
        typeof metadata !== "string" &&
        typeof metadata !== "object" &&
        metadata != null
    ) {
        throw Error(
            `Expected metadata param to be either a string or an object but found ${metadata}`
        );
    }
}

function checkApprover(approver: PlatformAddressValue | null) {
    if (approver != null && !PlatformAddress.check(approver)) {
        throw Error(
            `Expected approver param to be either null or a PlatformAddress value but found ${approver}`
        );
    }
}

function checkregistrar(registrar: PlatformAddressValue | null) {
    if (registrar != null && !PlatformAddress.check(registrar)) {
        throw Error(
            `Expected registrar param to be either null or a PlatformAddress value but found ${registrar}`
        );
    }
}

function checkCertifier(certifier: PlatformAddressValue) {
    if (!PlatformAddress.check(certifier)) {
        throw Error(
            `Expected certifier param to be a PlatformAddress but found ${certifier}`
        );
    }
}

function checkPayer(payer: PlatformAddressValue) {
    if (!PlatformAddress.check(payer)) {
        throw Error(
            `Expected payer param to be a PlatformAddress but found ${payer}`
        );
    }
}

function checkOwners(owners: Array<PlatformAddressValue>) {
    if (!Array.isArray(owners)) {
        throw Error(`Expected owners param to be an array but found ${owners}`);
    }
    owners.forEach((owner, index) => {
        if (!PlatformAddress.check(owner)) {
            throw Error(
                `Expected an owner address to be a PlatformAddress value but found ${owner} at index ${index}`
            );
        }
    });
}

function checkUsers(users: Array<PlatformAddressValue>) {
    if (!Array.isArray(users)) {
        throw Error(`Expected users param to be an array but found ${users}`);
    }
    users.forEach((user, index) => {
        if (!PlatformAddress.check(user)) {
            throw Error(
                `Expected a user address to be a PlatformAddress value but found ${user} at index ${index}`
            );
        }
    });
}

function checkTransferBurns(burns: Array<AssetTransferInput>) {
    if (!Array.isArray(burns)) {
        throw Error(`Expected burns param to be an array but found ${burns}`);
    }
    burns.forEach((burn, index) => {
        if (!(burn instanceof AssetTransferInput)) {
            throw Error(
                `Expected an item of burns to be an AssetTransferInput but found ${burn} at index ${index}`
            );
        }
    });
}

function checkTransferInputs(inputs: Array<AssetTransferInput>) {
    if (!Array.isArray(inputs)) {
        throw Error(`Expected inputs param to be an array but found ${inputs}`);
    }
    inputs.forEach((input, index) => {
        if (!(input instanceof AssetTransferInput)) {
            throw Error(
                `Expected an item of inputs to be an AssetTransferInput but found ${input} at index ${index}`
            );
        }
    });
}

function checkTransferOutputs(outputs: Array<AssetTransferOutput>) {
    if (!Array.isArray(outputs)) {
        throw Error(
            `Expected outputs param to be an array but found ${outputs}`
        );
    }
    outputs.forEach((output, index) => {
        if (!(output instanceof AssetTransferOutput)) {
            throw Error(
                `Expected an item of outputs to be an AssetTransferOutput but found ${output} at index ${index}`
            );
        }
    });
}

function checkTracker(value: H256Value) {
    if (!H256.check(value)) {
        throw Error(
            `Expected tracker param to be an H256 value but found ${value}`
        );
    }
}

function checkIndex(index: number) {
    if (typeof index !== "number") {
        throw Error(`Expected index param to be a number but found ${index}`);
    }
}

function checkAssetType(value: H160Value) {
    if (!H160.check(value)) {
        throw Error(
            `Expected assetType param to be an H160 value but found ${value}`
        );
    }
}

function checkAssetOutPoint(
    value:
        | AssetOutPoint
        | {
              tracker: H256Value;
              index: number;
              assetType: H160Value;
              shardId: number;
              quantity: U64Value;
              lockScriptHash?: H256Value;
              parameters?: Buffer[];
          }
) {
    if (value != null && typeof value !== "object") {
        throw Error(
            `Expected assetOutPoint param to be either an AssetOutPoint or an object but found ${value}`
        );
    }
    const {
        tracker,
        index,
        assetType,
        shardId,
        quantity,
        lockScriptHash,
        parameters
    } = value;
    checkTracker(tracker);
    checkIndex(index);
    checkAssetType(assetType);
    checkShardId(shardId);
    checkAmount(quantity);
    if (lockScriptHash) {
        checkLockScriptHash(lockScriptHash);
    }
    if (parameters) {
        checkParameters(parameters);
    }
}

function checkOrder(order: Order | null) {
    if (order != null && !(order instanceof Order)) {
        throw Error(
            `Expected order param to be either null or an Order value but found ${order}`
        );
    }
}

function checkIndices(indices: Array<number>) {
    if (!Array.isArray(indices)) {
        throw Error(
            `Expected indices param to be an array but found ${indices}`
        );
    }
    indices.forEach((value, idx) => {
        if (typeof value !== "number") {
            throw Error(
                `Expected an indices to be an array of numbers but found ${value} at index ${idx}`
            );
        }
    });
}

function checkLockScriptHash(value: H160Value) {
    if (!H160.check(value)) {
        throw Error(
            `Expected lockScriptHash param to be an H160 value but found ${value}`
        );
    }
}

function checkTransactionHash(value: H256Value) {
    if (!H256.check(value)) {
        throw Error(
            `Expected hash param to be an H256 value but found ${value}`
        );
    }
}

function checkSecret(value: H256Value) {
    if (!H256.check(value)) {
        throw Error(
            `Expected secret param to be an H256 value but found ${value}`
        );
    }
}

function checkParameters(parameters: Buffer[]) {
    if (!Array.isArray(parameters)) {
        throw Error(
            `Expected parameters param to be an array but found ${parameters}`
        );
    }
    parameters.forEach((p, index) => {
        if (!(p instanceof Buffer)) {
            throw Error(
                `Expected an item of parameters to be a Buffer instance but found ${p} at index ${index}`
            );
        }
    });
}

function checkTimelock(timelock: Timelock | null) {
    if (timelock == null) {
        return;
    }
    const { type, value } = timelock;
    if (
        type === "block" ||
        type === "blockAge" ||
        type === "time" ||
        type === "timeAge"
    ) {
        return;
    }
    if (typeof value === "number") {
        return;
    }
    throw Error(
        `Expected timelock param to be either null or an object containing both type and value but found ${timelock}`
    );
}

function checkLockScript(lockScript: Buffer) {
    if (!(lockScript instanceof Buffer)) {
        throw Error(
            `Expedted lockScript param to be an instance of Buffer but found ${lockScript}`
        );
    }
}

function checkUnlockScript(unlockScript: Buffer) {
    if (!(unlockScript instanceof Buffer)) {
        throw Error(
            `Expected unlockScript param to be an instance of Buffer but found ${unlockScript}`
        );
    }
}

function checkSignature(signature: string) {
    // Ed25519 Signature
    if (
        typeof signature !== "string" ||
        !/^(0x)?[0-9a-fA-F]{130}$/.test(signature)
    ) {
        throw Error(
            `Expected signature param to be a 65 byte hexstring but found ${signature}`
        );
    }
}

function checkHandlerId(handlerId: number) {
    if (
        typeof handlerId !== "number" ||
        !Number.isInteger(handlerId) ||
        handlerId < 0
    ) {
        throw Error(
            `Expected handlerId param to be a non-negative number value but found ${handlerId}`
        );
    }
}

function checkBytes(bytes: Buffer) {
    if (!(bytes instanceof Buffer)) {
        throw Error(
            `Expected bytes param to be an instance of Buffer but found ${bytes}`
        );
    }
}
