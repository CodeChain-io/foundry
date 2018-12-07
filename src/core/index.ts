import { AssetTransferAddress, PlatformAddress } from "codechain-primitives";

import { AssetTransaction } from "./action/AssetTransaction";
import { CreateShard } from "./action/CreateShard";
import { Payment } from "./action/Payment";
import { SetRegularKey } from "./action/SetReulgarKey";
import { SetShardOwners } from "./action/SetShardOwners";
import { SetShardUsers } from "./action/SetShardUsers";
import { WrapCCC } from "./action/WrapCCC";
import { Asset } from "./Asset";
import { AssetScheme } from "./AssetScheme";
import { Block } from "./Block";
import { H128 } from "./H128";
import { H160 } from "./H160";
import { H256 } from "./H256";
import { H512 } from "./H512";
import { Invoice } from "./Invoice";
import { Parcel } from "./Parcel";
import { Script } from "./Script";
import { SignedParcel } from "./SignedParcel";
import { AssetComposeTransaction } from "./transaction/AssetComposeTransaction";
import { AssetDecomposeTransaction } from "./transaction/AssetDecomposeTransaction";
import { AssetMintOutput } from "./transaction/AssetMintOutput";
import { AssetMintTransaction } from "./transaction/AssetMintTransaction";
import { AssetOutPoint } from "./transaction/AssetOutPoint";
import { AssetTransferInput, Timelock } from "./transaction/AssetTransferInput";
import { AssetTransferOutput } from "./transaction/AssetTransferOutput";
import { AssetTransferTransaction } from "./transaction/AssetTransferTransaction";
import { AssetUnwrapCCCTransaction } from "./transaction/AssetUnwrapCCCTransaction";
import { Order } from "./transaction/Order";
import { OrderOnTransfer } from "./transaction/OrderOnTransfer";
import { getTransactionFromJSON, Transaction } from "./transaction/Transaction";
import { NetworkId } from "./types";
import { U256 } from "./U256";
import { U64 } from "./U64";

export class Core {
    public static classes = {
        // Data
        H128,
        H160,
        H256,
        H512,
        U256,
        U64,
        Invoice,
        // Block
        Block,
        // Parcel
        Parcel,
        SignedParcel,
        // Action
        Payment,
        SetRegularKey,
        AssetTransaction,
        CreateShard,
        SetShardOwners,
        SetShardUsers,
        WrapCCC,
        // Transaction
        AssetMintTransaction,
        AssetTransferTransaction,
        AssetComposeTransaction,
        AssetDecomposeTransaction,
        AssetUnwrapCCCTransaction,
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
        AssetTransferAddress
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
     * Creates Payment action which pays the value amount of CCC(CodeChain Coin)
     * from the parcel signer to the recipient. Who is signing the parcel will pay.
     * @param params.recipient The platform account who receives CCC
     * @param params.amount Amount of CCC to pay
     * @throws Given string for recipient is invalid for converting it to PlatformAddress
     * @throws Given number or string for amount is invalid for converting it to U64
     */
    public createPaymentParcel(params: {
        recipient: PlatformAddress | string;
        amount: U64 | number | string;
    }): Parcel {
        const { recipient, amount } = params;
        checkPlatformAddressRecipient(recipient);
        checkAmount(amount);
        return new Parcel(
            this.networkId,
            new Payment(PlatformAddress.ensure(recipient), U64.ensure(amount))
        );
    }

    /**
     * Creates SetRegularKey action which sets the regular key of the parcel signer.
     * @param params.key The public key of a regular key
     * @throws Given string for key is invalid for converting it to H512
     */
    public createSetRegularKeyParcel(params: { key: H512 | string }): Parcel {
        const { key } = params;
        checkKey(key);
        return new Parcel(this.networkId, new SetRegularKey(H512.ensure(key)));
    }

    /**
     * Creates AssetTransaction action which can mint or transfer assets through
     * AssetMintTransaction or AssetTransferTransaction.
     * @param params.transaction Transaction
     */
    public createAssetTransactionParcel(params: {
        transaction: Transaction;
        approvals?: string[];
    }): Parcel {
        const { transaction, approvals = [] } = params;
        checkTransaction(transaction);
        return new Parcel(
            this.networkId,
            new AssetTransaction({ transaction, approvals })
        );
    }

    /**
     * Creates CreateShard action which can create new shard
     */
    public createCreateShardParcel(): Parcel {
        return new Parcel(this.networkId, new CreateShard());
    }

    public createSetShardOwnersParcel(params: {
        shardId: number;
        owners: Array<PlatformAddress | string>;
    }): Parcel {
        const { shardId, owners } = params;
        checkShardId(shardId);
        checkOwners(owners);
        return new Parcel(
            this.networkId,
            new SetShardOwners({
                shardId,
                owners: owners.map(PlatformAddress.ensure)
            })
        );
    }

    /**
     * Create SetShardUser action which can change shard users
     * @param params.shardId
     * @param params.users
     */
    public createSetShardUsersParcel(params: {
        shardId: number;
        users: Array<PlatformAddress | string>;
    }): Parcel {
        const { shardId, users } = params;
        checkShardId(shardId);
        checkUsers(users);
        return new Parcel(
            this.networkId,
            new SetShardUsers({
                shardId,
                users: users.map(PlatformAddress.ensure)
            })
        );
    }

    /**
     * Creates Wrap CCC action which wraps the value amount of CCC(CodeChain Coin)
     * in a wrapped CCC asset. Who is signing the parcel will pay.
     * @param params.shardId A shard ID of the wrapped CCC asset.
     * @param params.lockScriptHash A lock script hash of the wrapped CCC asset.
     * @param params.parameters Parameters of the wrapped CCC asset.
     * @param params.amount Amount of CCC to pay
     * @throws Given string for a lock script hash is invalid for converting it to H160
     * @throws Given number or string for amount is invalid for converting it to U64
     */
    public createWrapCCCParcel(
        params:
            | {
                  shardId: number;
                  lockScriptHash: H160 | string;
                  parameters: Buffer[];
                  amount: U64 | number | string;
              }
            | {
                  shardId: number;
                  recipient: AssetTransferAddress | string;
                  amount: U64 | number | string;
              }
    ): Parcel {
        const { shardId, amount } = params;
        checkShardId(shardId);
        checkAmount(amount);
        if ("recipient" in params) {
            checkAssetTransferAddressRecipient(params.recipient);
            return new Parcel(
                this.networkId,
                new WrapCCC({
                    shardId,
                    recipient: AssetTransferAddress.ensure(params.recipient),
                    amount: U64.ensure(amount)
                })
            );
        } else {
            const { lockScriptHash, parameters } = params;
            checkLockScriptHash(lockScriptHash);
            checkParameters(parameters);
            return new Parcel(
                this.networkId,
                new WrapCCC({
                    shardId,
                    lockScriptHash: H160.ensure(lockScriptHash),
                    parameters,
                    amount: U64.ensure(amount)
                })
            );
        }
    }

    /**
     * Creates asset's scheme.
     * @param params.metadata Any string that describing the asset. For example,
     * stringified JSON containing properties.
     * @param params.amount Total amount of this asset
     * @param params.approver Platform account or null. If account is present, the
     * parcel that includes AssetTransferTransaction of this asset must be signed by
     * the approver account.
     * @param params.administrator Platform account or null. The administrator
     * can transfer the asset without unlocking.
     * @throws Given string for approver is invalid for converting it to paltform account
     * @throws Given string for administrator is invalid for converting it to paltform account
     */
    public createAssetScheme(params: {
        shardId: number;
        metadata: string;
        amount: U64 | number | string;
        approver?: PlatformAddress | string;
        administrator?: PlatformAddress | string;
        pool?: { assetType: H256 | string; amount: number }[];
    }): AssetScheme {
        const {
            shardId,
            metadata,
            amount,
            approver = null,
            administrator = null,
            pool = []
        } = params;
        checkShardId(shardId);
        checkMetadata(metadata);
        checkAmount(amount);
        checkApprover(approver);
        checkAdministrator(administrator);
        return new AssetScheme({
            networkId: this.networkId,
            shardId,
            metadata,
            amount: U64.ensure(amount),
            approver:
                approver == null ? null : PlatformAddress.ensure(approver),
            administrator:
                administrator == null
                    ? null
                    : PlatformAddress.ensure(administrator),
            pool: pool.map(({ assetType, amount: assetAmount }) => ({
                assetType: H256.ensure(assetType),
                amount: U64.ensure(assetAmount)
            }))
        });
    }

    public createOrder(
        params: {
            assetTypeFrom: H256 | string;
            assetTypeTo: H256 | string;
            assetTypeFee?: H256 | string;
            assetAmountFrom: U64 | number | string;
            assetAmountTo: U64 | number | string;
            assetAmountFee?: U64 | number | string;
            originOutputs:
                | AssetOutPoint[]
                | {
                      transactionHash: H256 | string;
                      index: number;
                      assetType: H256 | string;
                      amount: U64 | number | string;
                      lockScriptHash?: H256 | string;
                      parameters?: Buffer[];
                  }[];
            expiration: U64 | number | string;
        } & (
            | {
                  lockScriptHash: H160 | string;
                  parameters: Buffer[];
              }
            | {
                  recipient: AssetTransferAddress | string;
              })
    ): Order {
        const {
            assetTypeFrom,
            assetTypeTo,
            assetTypeFee = "0000000000000000000000000000000000000000000000000000000000000000",
            assetAmountFrom,
            assetAmountTo,
            assetAmountFee = 0,
            originOutputs,
            expiration
        } = params;
        checkAssetType(assetTypeFrom);
        checkAssetType(assetTypeTo);
        checkAssetType(assetTypeFee);
        checkAmount(assetAmountFrom);
        checkAmount(assetAmountTo);
        checkAmount(assetAmountFee);
        checkExpiration(expiration);
        const originOutputsConv: AssetOutPoint[] = [];
        for (let i = 0; i < originOutputs.length; i++) {
            const originOutput = originOutputs[i];
            const {
                transactionHash,
                index,
                assetType,
                amount,
                lockScriptHash,
                parameters
            } = originOutput;
            checkAssetOutPoint(originOutput);
            originOutputsConv[i] =
                originOutput instanceof AssetOutPoint
                    ? originOutput
                    : new AssetOutPoint({
                          transactionHash: H256.ensure(transactionHash),
                          index,
                          assetType: H256.ensure(assetType),
                          amount: U64.ensure(amount),
                          lockScriptHash: lockScriptHash
                              ? H160.ensure(lockScriptHash)
                              : undefined,
                          parameters
                      });
        }

        if ("recipient" in params) {
            checkAssetTransferAddressRecipient(params.recipient);
            return new Order({
                assetTypeFrom: H256.ensure(assetTypeFrom),
                assetTypeTo: H256.ensure(assetTypeTo),
                assetTypeFee: H256.ensure(assetTypeFee),
                assetAmountFrom: U64.ensure(assetAmountFrom),
                assetAmountTo: U64.ensure(assetAmountTo),
                assetAmountFee: U64.ensure(assetAmountFee),
                expiration: U64.ensure(expiration),
                originOutputs: originOutputsConv,
                recipient: AssetTransferAddress.ensure(params.recipient)
            });
        } else {
            const { lockScriptHash, parameters } = params;
            checkLockScriptHash(lockScriptHash);
            checkParameters(parameters);
            return new Order({
                assetTypeFrom: H256.ensure(assetTypeFrom),
                assetTypeTo: H256.ensure(assetTypeTo),
                assetTypeFee: H256.ensure(assetTypeFee),
                assetAmountFrom: U64.ensure(assetAmountFrom),
                assetAmountTo: U64.ensure(assetAmountTo),
                assetAmountFee: U64.ensure(assetAmountFee),
                expiration: U64.ensure(expiration),
                originOutputs: originOutputsConv,
                lockScriptHash: H160.ensure(lockScriptHash),
                parameters
            });
        }
    }
    public createOrderOnTransfer(params: {
        order: Order;
        spentAmount: U64 | string | number;
        inputIndices: number[];
        outputIndices: number[];
    }) {
        const { order, spentAmount, inputIndices, outputIndices } = params;
        checkOrder(order);
        checkAmount(spentAmount);
        checkIndices(inputIndices);
        checkIndices(outputIndices);

        return new OrderOnTransfer({
            order,
            spentAmount: U64.ensure(spentAmount),
            inputIndices,
            outputIndices
        });
    }

    public createAssetMintTransaction(params: {
        scheme:
            | AssetScheme
            | {
                  networkId?: NetworkId;
                  shardId: number;
                  metadata: string;
                  approver?: PlatformAddress | string;
                  administrator?: PlatformAddress | string;
                  amount?: U64 | number | string | null;
              };
        recipient: AssetTransferAddress | string;
    }): AssetMintTransaction {
        const { scheme, recipient } = params;
        if (scheme !== null && typeof scheme !== "object") {
            throw Error(
                `Expected scheme param to be either an AssetScheme or an object but found ${scheme}`
            );
        }
        const {
            networkId = this.networkId,
            shardId,
            metadata,
            approver: approver = null,
            administrator: administrator = null,
            amount
        } = scheme;
        checkAssetTransferAddressRecipient(recipient);
        checkNetworkId(networkId);
        if (shardId === undefined) {
            throw Error(`shardId is undefined`);
        }
        checkShardId(shardId);
        checkMetadata(metadata);
        checkApprover(approver);
        checkAdministrator(administrator);
        if (amount != null) {
            checkAmount(amount);
        }
        return new AssetMintTransaction({
            networkId,
            shardId,
            approver:
                approver == null ? null : PlatformAddress.ensure(approver),
            administrator:
                administrator == null
                    ? null
                    : PlatformAddress.ensure(administrator),
            metadata,
            output: new AssetMintOutput({
                amount: amount == null ? null : U64.ensure(amount),
                recipient: AssetTransferAddress.ensure(recipient)
            })
        });
    }

    public createAssetTransferTransaction(params?: {
        burns?: AssetTransferInput[];
        inputs?: AssetTransferInput[];
        outputs?: AssetTransferOutput[];
        orders?: OrderOnTransfer[];
        networkId?: NetworkId;
    }): AssetTransferTransaction {
        const {
            burns = [],
            inputs = [],
            outputs = [],
            orders = [],
            networkId = this.networkId
        } = params || {};
        checkTransferBurns(burns);
        checkTransferInputs(inputs);
        checkTransferOutputs(outputs);
        checkNetworkId(networkId);
        return new AssetTransferTransaction({
            burns,
            inputs,
            outputs,
            orders,
            networkId
        });
    }

    public createAssetComposeTransaction(params: {
        scheme:
            | AssetScheme
            | {
                  shardId: number;
                  metadata: string;
                  amount?: U64 | number | string | null;
                  approver?: PlatformAddress | string;
                  administrator?: PlatformAddress | string;
                  networkId?: NetworkId;
              };
        inputs: AssetTransferInput[];
        recipient: AssetTransferAddress | string;
    }): AssetComposeTransaction {
        const { scheme, inputs, recipient } = params;
        const {
            networkId = this.networkId,
            shardId,
            metadata,
            approver = null,
            administrator = null,
            amount
        } = scheme;
        checkTransferInputs(inputs);
        checkAssetTransferAddressRecipient(recipient);
        checkNetworkId(networkId);
        if (shardId === undefined) {
            throw Error(`shardId is undefined`);
        }
        checkShardId(shardId);
        checkMetadata(metadata);
        checkApprover(approver);
        if (amount != null) {
            checkAmount(amount);
        }
        return new AssetComposeTransaction({
            networkId,
            shardId,
            approver:
                approver === null ? null : PlatformAddress.ensure(approver),
            administrator:
                administrator === null
                    ? null
                    : PlatformAddress.ensure(administrator),
            metadata,
            inputs,
            output: new AssetMintOutput({
                recipient: AssetTransferAddress.ensure(recipient),
                amount: amount == null ? null : U64.ensure(amount)
            })
        });
    }

    public createAssetDecomposeTransaction(params: {
        input: AssetTransferInput;
        outputs?: AssetTransferOutput[];
        networkId?: NetworkId;
    }): AssetDecomposeTransaction {
        if (
            params === null ||
            typeof params !== "object" ||
            !("input" in params)
        ) {
            throw Error(
                `Expected the first param of createAssetDecomposeTransaction to be an object containing input param but found ${params}`
            );
        }
        const { input, outputs = [], networkId = this.networkId } = params;
        checkTransferInput(input);
        checkTransferOutputs(outputs);
        checkNetworkId(networkId);
        return new AssetDecomposeTransaction({
            input,
            outputs,
            networkId
        });
    }

    public createAssetUnwrapCCCTransaction(params: {
        burn: AssetTransferInput | Asset;
        networkId?: NetworkId;
    }): AssetUnwrapCCCTransaction {
        const { burn, networkId = this.networkId } = params;
        checkNetworkId(networkId);
        if (burn instanceof Asset) {
            const burnInput = burn.createTransferInput();
            checkTransferBurns([burnInput]);
            return new AssetUnwrapCCCTransaction({
                burn: burnInput,
                networkId
            });
        } else {
            checkTransferBurns([burn]);
            return new AssetUnwrapCCCTransaction({
                burn,
                networkId
            });
        }
    }

    public createAssetTransferInput(params: {
        assetOutPoint:
            | AssetOutPoint
            | {
                  transactionHash: H256 | string;
                  index: number;
                  assetType: H256 | string;
                  amount: U64 | number | string;
                  lockScriptHash?: H256 | string;
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
            transactionHash,
            index,
            assetType,
            amount,
            lockScriptHash,
            parameters
        } = assetOutPoint;
        return new AssetTransferInput({
            prevOut:
                assetOutPoint instanceof AssetOutPoint
                    ? assetOutPoint
                    : new AssetOutPoint({
                          transactionHash: H256.ensure(transactionHash),
                          index,
                          assetType: H256.ensure(assetType),
                          amount: U64.ensure(amount),
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
        transactionHash: H256 | string;
        index: number;
        assetType: H256 | string;
        amount: U64 | number | string;
    }): AssetOutPoint {
        const { transactionHash, index, assetType, amount } = params;
        checkTransactionHash(transactionHash);
        checkIndex(index);
        checkAssetType(assetType);
        checkAmount(amount);
        return new AssetOutPoint({
            transactionHash: H256.ensure(transactionHash),
            index,
            assetType: H256.ensure(assetType),
            amount: U64.ensure(amount)
        });
    }

    public createAssetTransferOutput(
        params: {
            assetType: H256 | string;
            amount: U64 | number | string;
        } & (
            | {
                  recipient: AssetTransferAddress | string;
              }
            | {
                  lockScriptHash: H256 | string;
                  parameters: Buffer[];
              })
    ): AssetTransferOutput {
        const { assetType } = params;
        const amount = U64.ensure(params.amount);
        checkAssetType(assetType);
        checkAmount(amount);
        if ("recipient" in params) {
            const { recipient } = params;
            checkAssetTransferAddressRecipient(recipient);
            return new AssetTransferOutput({
                recipient: AssetTransferAddress.ensure(recipient),
                assetType: H256.ensure(assetType),
                amount
            });
        } else if ("lockScriptHash" in params && "parameters" in params) {
            const { lockScriptHash, parameters } = params;
            checkLockScriptHash(lockScriptHash);
            checkParameters(parameters);
            return new AssetTransferOutput({
                lockScriptHash: H160.ensure(lockScriptHash),
                parameters,
                assetType: H256.ensure(assetType),
                amount
            });
        } else {
            throw Error(`Unexpected params: ${params}`);
        }
    }

    // FIXME: any
    public getTransactionFromJSON(json: any): Transaction {
        return getTransactionFromJSON(json);
    }
}

function checkNetworkId(networkId: NetworkId) {
    if (typeof networkId !== "string" || networkId.length !== 2) {
        throw Error(
            `Expected networkId param to be a string of length 2 but found ${networkId}`
        );
    }
}

function checkPlatformAddressRecipient(recipient: PlatformAddress | string) {
    if (!PlatformAddress.check(recipient)) {
        throw Error(
            `Expected recipient param to be a PlatformAddress but found ${recipient}`
        );
    }
}

function checkAssetTransferAddressRecipient(
    recipient: AssetTransferAddress | string
) {
    if (!AssetTransferAddress.check(recipient)) {
        throw Error(
            `Expected recipient param to be a AssetTransferAddress but found ${recipient}`
        );
    }
}

function checkAmount(amount: U64 | number | string) {
    if (!U64.check(amount)) {
        throw Error(
            `Expected amount param to be a U64 value but found ${amount}`
        );
    }
}

function checkExpiration(expiration: U64 | number | string) {
    if (!U64.check(expiration)) {
        throw Error(
            `Expected expiration param to be a U64 value but found ${expiration}`
        );
    }
}

function checkKey(key: H512 | string) {
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

function checkMetadata(metadata: string) {
    if (typeof metadata !== "string") {
        throw Error(
            `Expected metadata param to be a string but found ${metadata}`
        );
    }
}

function checkApprover(approver: PlatformAddress | string | null) {
    if (approver !== null && !PlatformAddress.check(approver)) {
        throw Error(
            `Expected approver param to be either null or a PlatformAddress value but found ${approver}`
        );
    }
}

function checkAdministrator(administrator: PlatformAddress | string | null) {
    if (administrator !== null && !PlatformAddress.check(administrator)) {
        throw Error(
            `Expected administrator param to be either null or a PlatformAddress value but found ${administrator}`
        );
    }
}

function checkTransaction(_transaction: Transaction) {
    // FIXME: check whether the transaction is valid
}

function checkOwners(owners: Array<PlatformAddress | string>) {
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

function checkUsers(users: Array<PlatformAddress | string>) {
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

function checkTransferInput(input: AssetTransferInput) {
    if (!(input instanceof AssetTransferInput)) {
        throw Error(
            `Expected an input param to be an AssetTransferInput but found ${input}`
        );
    }
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

function checkTransactionHash(value: H256 | string) {
    if (!H256.check(value)) {
        throw Error(
            `Expected transactionHash param to be an H256 value but found ${value}`
        );
    }
}

function checkIndex(index: number) {
    if (typeof index !== "number") {
        throw Error(`Expected index param to be a number but found ${index}`);
    }
}

function checkAssetType(value: H256 | string) {
    if (!H256.check(value)) {
        throw Error(
            `Expected assetType param to be an H256 value but found ${value}`
        );
    }
}

function checkAssetOutPoint(
    value:
        | AssetOutPoint
        | {
              transactionHash: H256 | string;
              index: number;
              assetType: H256 | string;
              amount: U64 | number | string;
              lockScriptHash?: H256 | string;
              parameters?: Buffer[];
          }
) {
    if (value !== null && typeof value !== "object") {
        throw Error(
            `Expected assetOutPoint param to be either an AssetOutPoint or an object but found ${value}`
        );
    }
    const {
        transactionHash,
        index,
        assetType,
        amount,
        lockScriptHash,
        parameters
    } = value;
    checkTransactionHash(transactionHash);
    checkIndex(index);
    checkAssetType(assetType);
    checkAmount(amount);
    if (lockScriptHash) {
        checkLockScriptHash(lockScriptHash);
    }
    if (parameters) {
        checkParameters(parameters);
    }
}

function checkOrder(order: Order | null) {
    if (order !== null && !(order instanceof Order)) {
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

function checkLockScriptHash(value: H160 | string) {
    if (!H160.check(value)) {
        throw Error(
            `Expected lockScriptHash param to be an H160 value but found ${value}`
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
    if (timelock === null) {
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
