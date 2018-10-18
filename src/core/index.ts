import { AssetTransferAddress, PlatformAddress } from "codechain-primitives";

import { AssetTransactionGroup } from "./action/AssetTransactionGroup";
import { CreateShard } from "./action/CreateShard";
import { Payment } from "./action/Payment";
import { SetRegularKey } from "./action/SetReulgarKey";
import { SetShardOwners } from "./action/SetShardOwners";
import { SetShardUsers } from "./action/SetShardUsers";
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
import { getTransactionFromJSON, Transaction } from "./transaction/Transaction";
import { NetworkId } from "./types";
import { U256 } from "./U256";

export class Core {
    public static classes = {
        // Data
        H128,
        H160,
        H256,
        H512,
        U256,
        Invoice,
        // Block
        Block,
        // Parcel
        Parcel,
        SignedParcel,
        // Action
        Payment,
        SetRegularKey,
        AssetTransactionGroup,
        CreateShard,
        SetShardOwners,
        SetShardUsers,
        // Transaction
        AssetMintTransaction,
        AssetTransferTransaction,
        AssetComposeTransaction,
        AssetDecomposeTransaction,
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
     * @throws Given number or string for amount is invalid for converting it to U256
     */
    public createPaymentParcel(params: {
        recipient: PlatformAddress | string;
        amount: U256 | number | string;
    }): Parcel {
        const { recipient, amount } = params;
        checkPlatformAddressRecipient(recipient);
        checkAmount(amount);
        return new Parcel(
            this.networkId,
            new Payment(PlatformAddress.ensure(recipient), U256.ensure(amount))
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
     * Creates AssetTransactionGroup action which can mint or transfer assets through
     * AssetMintTransaction or AssetTransferTransaction.
     * @param params.transactions List of transaction
     */
    public createAssetTransactionGroupParcel(params: {
        transactions: Transaction[];
    }): Parcel {
        const { transactions } = params;
        checkTransactions(transactions);
        return new Parcel(
            this.networkId,
            new AssetTransactionGroup({ transactions })
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
     * Creates asset's scheme.
     * @param params.metadata Any string that describing the asset. For example,
     * stringified JSON containing properties.
     * @param params.amount Total amount of this asset
     * @param params.registrar Platform account or null. If account is present, the
     * parcel that includes AssetTransferTransaction of this asset must be signed by
     * the registrar account.
     * @throws Given string for registrar is invalid for converting it to paltform account
     */
    public createAssetScheme(params: {
        shardId: number;
        metadata: string;
        amount: number;
        registrar?: PlatformAddress | string;
        pool?: { assetType: H256 | string; amount: number }[];
    }): AssetScheme {
        const {
            shardId,
            metadata,
            amount,
            registrar = null,
            pool = []
        } = params;
        checkShardId(shardId);
        checkMetadata(metadata);
        checkAmountU64(amount);
        checkRegistrar(registrar);
        return new AssetScheme({
            networkId: this.networkId,
            shardId,
            metadata,
            amount,
            registrar:
                registrar === null ? null : PlatformAddress.ensure(registrar),
            pool: pool.map(({ assetType, amount: assetAmount }) => ({
                assetType: H256.ensure(assetType),
                amount: assetAmount
            }))
        });
    }

    public createAssetMintTransaction(params: {
        scheme:
            | AssetScheme
            | {
                  networkId?: NetworkId;
                  shardId: number;
                  metadata: string;
                  registrar?: PlatformAddress | string;
                  amount: number | null;
              };
        recipient: AssetTransferAddress | string;
        nonce?: number;
    }): AssetMintTransaction {
        const { scheme, recipient, nonce = 0 } = params;
        if (scheme !== null && typeof scheme !== "object") {
            throw Error(
                `Expected scheme param to be either an AssetScheme or an object but found ${scheme}`
            );
        }
        const {
            networkId = this.networkId,
            shardId,
            metadata,
            registrar = null,
            amount
        } = scheme;
        checkAssetTransferAddressRecipient(recipient);
        checkNonce(nonce);
        checkNetworkId(networkId);
        if (shardId === undefined) {
            throw Error(`shardId is undefined`);
        }
        checkShardId(shardId);
        checkMetadata(metadata);
        checkRegistrar(registrar);
        if (amount !== null) {
            checkAmountU64(amount);
        }
        return new AssetMintTransaction({
            networkId,
            shardId,
            nonce,
            registrar:
                registrar == null ? null : PlatformAddress.ensure(registrar),
            metadata,
            output: new AssetMintOutput({
                amount,
                recipient: AssetTransferAddress.ensure(recipient)
            })
        });
    }

    public createAssetTransferTransaction(params?: {
        burns?: AssetTransferInput[];
        inputs?: AssetTransferInput[];
        outputs?: AssetTransferOutput[];
        networkId?: NetworkId;
        nonce?: number;
    }): AssetTransferTransaction {
        const {
            burns = [],
            inputs = [],
            outputs = [],
            networkId = this.networkId,
            nonce = 0
        } = params || {};
        checkTransferBurns(burns);
        checkTransferInputs(inputs);
        checkTransferOutputs(outputs);
        checkNetworkId(networkId);
        checkNonce(nonce);
        return new AssetTransferTransaction({
            burns,
            inputs,
            outputs,
            networkId,
            nonce
        });
    }

    public createAssetComposeTransaction(params: {
        scheme:
            | AssetScheme
            | {
                  shardId: number;
                  metadata: string;
                  amount: number | null;
                  registrar?: PlatformAddress | string;
                  networkId?: NetworkId;
              };
        inputs: AssetTransferInput[];
        recipient: AssetTransferAddress | string;
        nonce?: number;
    }): AssetComposeTransaction {
        const { scheme, inputs, recipient, nonce = 0 } = params;
        const {
            networkId = this.networkId,
            shardId,
            metadata,
            registrar = null,
            amount
        } = scheme;
        checkTransferInputs(inputs);
        checkAssetTransferAddressRecipient(recipient);
        checkNonce(nonce);
        checkNetworkId(networkId);
        if (shardId === undefined) {
            throw Error(`shardId is undefined`);
        }
        checkShardId(shardId);
        checkMetadata(metadata);
        checkRegistrar(registrar);
        if (amount !== null) {
            checkAmountU64(amount);
        }
        return new AssetComposeTransaction({
            networkId,
            shardId,
            nonce,
            registrar:
                registrar === null ? null : PlatformAddress.ensure(registrar),
            metadata,
            inputs,
            output: new AssetMintOutput({
                recipient: AssetTransferAddress.ensure(recipient),
                amount
            })
        });
    }

    public createAssetDecomposeTransaction(params: {
        input: AssetTransferInput;
        outputs?: AssetTransferOutput[];
        networkId?: NetworkId;
        nonce?: number;
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
        const {
            input,
            outputs = [],
            networkId = this.networkId,
            nonce = 0
        } = params;
        checkTransferInput(input);
        checkTransferOutputs(outputs);
        checkNetworkId(networkId);
        checkNonce(nonce);
        return new AssetDecomposeTransaction({
            input,
            outputs,
            networkId,
            nonce
        });
    }

    public createAssetTransferInput(params: {
        assetOutPoint:
            | AssetOutPoint
            | {
                  transactionHash: H256 | string;
                  index: number;
                  assetType: H256 | string;
                  amount: number;
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
        if (assetOutPoint !== null && typeof assetOutPoint !== "object") {
            throw Error(
                `Expected assetOutPoint param to be either an AssetOutPoint or an object but found ${assetOutPoint}`
            );
        }
        const {
            transactionHash,
            index,
            assetType,
            amount,
            lockScriptHash,
            parameters
        } = assetOutPoint;
        checkTransactionHash(transactionHash);
        checkIndex(index);
        checkAssetType(assetType);
        checkAmountU64(amount);
        checkTimelock(timelock);
        if (lockScriptHash) {
            checkLockScriptHash(lockScriptHash);
        }
        if (parameters) {
            checkParameters(parameters);
        }
        if (lockScript) {
            checkLockScript(lockScript);
        }
        if (unlockScript) {
            checkUnlockScript(unlockScript);
        }
        return new AssetTransferInput({
            prevOut:
                assetOutPoint instanceof AssetOutPoint
                    ? assetOutPoint
                    : new AssetOutPoint({
                          transactionHash: H256.ensure(transactionHash),
                          index,
                          assetType: H256.ensure(assetType),
                          amount,
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
        amount: number;
    }): AssetOutPoint {
        const { transactionHash, index, assetType, amount } = params;
        checkTransactionHash(transactionHash);
        checkIndex(index);
        checkAssetType(assetType);
        checkAmountU64(amount);
        return new AssetOutPoint({
            transactionHash: H256.ensure(transactionHash),
            index,
            assetType: H256.ensure(assetType),
            amount
        });
    }

    public createAssetTransferOutput(
        params: {
            assetType: H256 | string;
            amount: number;
        } & (
            | {
                  recipient: AssetTransferAddress | string;
              }
            | {
                  lockScriptHash: H256 | string;
                  parameters: Buffer[];
              })
    ): AssetTransferOutput {
        const { assetType, amount } = params;
        checkAssetType(assetType);
        checkAmountU64(amount);
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

function checkNonce(nonce: number) {
    if (typeof nonce !== "number") {
        throw Error(`Expected nonce param to be a number but found ${nonce}`);
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

function checkAmount(amount: U256 | number | string) {
    if (!U256.check(amount)) {
        throw Error(
            `Expected amount param to be a U256 value but found ${amount}`
        );
    }
}

// FIXME: U64
function checkAmountU64(amount: number) {
    if (typeof amount !== "number" || !Number.isInteger(amount) || amount < 0) {
        throw Error(`Expected amount param to be a number but found ${amount}`);
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

function checkRegistrar(registrar: PlatformAddress | string | null) {
    if (registrar !== null && !PlatformAddress.check(registrar)) {
        throw Error(
            `Expected registrar param to be either null or a PlatformAddress value but found ${registrar}`
        );
    }
}

function checkTransactions(transactions: Transaction[]) {
    if (!Array.isArray(transactions)) {
        throw Error(
            `Expected transactions param to be an array but found ${transactions}`
        );
    }
    // FIXME: check all transaction are valid
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
