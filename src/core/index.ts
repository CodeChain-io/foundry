// FIXME: Use interface instead of importing key class.
import { AssetTransferAddress, PlatformAddress } from "../key/classes";

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
import { AssetMintTransaction } from "./transaction/AssetMintTransaction";
import { AssetOutPoint } from "./transaction/AssetOutPoint";
import { AssetTransferInput } from "./transaction/AssetTransferInput";
import { AssetTransferOutput } from "./transaction/AssetTransferOutput";
import { AssetTransferTransaction } from "./transaction/AssetTransferTransaction";
import { CreateWorldTransaction } from "./transaction/CreateWorldTransaction";
import { SetWorldOwnersTransaction } from "./transaction/SetWorldOwnersTransaction";
import { SetWorldUsersTransaction } from "./transaction/SetWorldUsersTransaction";
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
        AssetTransferInput,
        AssetTransferOutput,
        AssetOutPoint,
        CreateWorldTransaction,
        SetWorldOwnersTransaction,
        SetWorldUsersTransaction,
        // Asset and AssetScheme
        Asset,
        AssetScheme,
        // Script
        Script
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
        worldId: number;
        metadata: string;
        amount: number;
        registrar?: PlatformAddress | string;
    }): AssetScheme {
        const { shardId, worldId, metadata, amount, registrar = null } = params;
        checkShardId(shardId);
        checkWorldId(worldId);
        checkMetadata(metadata);
        checkAmountU64(amount);
        checkRegistrar(registrar);
        return new AssetScheme({
            networkId: this.networkId,
            shardId,
            worldId,
            metadata,
            amount,
            registrar:
                registrar == null ? null : PlatformAddress.ensure(registrar)
        });
    }

    public createCreateWorldTransaction(params: {
        networkId?: NetworkId;
        shardId: number;
        owners: Array<PlatformAddress | string>;
        nonce?: number;
    }): CreateWorldTransaction {
        const {
            networkId = this.networkId,
            shardId,
            owners,
            nonce = 0
        } = params;
        checkNetworkId(networkId);
        checkShardId(shardId);
        checkOwners(owners);
        checkNonce(nonce);
        return new CreateWorldTransaction({
            networkId,
            shardId,
            owners: owners.map(PlatformAddress.ensure),
            nonce
        });
    }

    public createSetWorldOwnersTransaction(params: {
        networkId?: NetworkId;
        shardId: number;
        worldId: number;
        owners: Array<PlatformAddress | string>;
        nonce: number;
    }): SetWorldOwnersTransaction {
        const {
            networkId = this.networkId,
            shardId,
            worldId,
            owners,
            nonce
        } = params;
        checkNetworkId(networkId);
        checkShardId(shardId);
        checkWorldId(worldId);
        checkOwners(owners);
        checkNonce(nonce);
        return new SetWorldOwnersTransaction({
            networkId,
            shardId,
            worldId,
            owners: owners.map(PlatformAddress.ensure),
            nonce
        });
    }

    public createSetWorldUsersTransaction(params: {
        networkId?: NetworkId;
        shardId: number;
        worldId: number;
        users: Array<PlatformAddress | string>;
        nonce: number;
    }): SetWorldUsersTransaction {
        const {
            networkId = this.networkId,
            shardId,
            worldId,
            users,
            nonce
        } = params;
        checkNetworkId(networkId);
        checkShardId(shardId);
        checkWorldId(worldId);
        checkUsers(users);
        checkNonce(nonce);
        return new SetWorldUsersTransaction({
            networkId,
            shardId,
            worldId,
            users: users.map(PlatformAddress.ensure),
            nonce
        });
    }

    public createAssetMintTransaction(params: {
        scheme:
            | AssetScheme
            | {
                  networkId?: NetworkId;
                  shardId: number;
                  worldId: number;
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
            worldId,
            metadata,
            registrar = null,
            amount
        } = scheme;
        checkAssetTransferAddressRecipient(recipient);
        checkNonce(nonce);
        checkNetworkId(networkId);
        checkShardId(shardId);
        checkWorldId(worldId);
        checkMetadata(metadata);
        checkRegistrar(registrar);
        return new AssetMintTransaction({
            networkId,
            shardId,
            worldId,
            nonce,
            registrar:
                registrar == null ? null : PlatformAddress.ensure(registrar),
            metadata,
            output: {
                amount,
                ...AssetTransferAddress.ensure(
                    recipient
                ).getLockScriptHashAndParameters()
            }
        });
    }

    public createAssetTransferTransaction(
        params: {
            burns: AssetTransferInput[];
            inputs: AssetTransferInput[];
            outputs: AssetTransferOutput[];
            networkId?: NetworkId;
            nonce?: number;
        } = { burns: [], inputs: [], outputs: [] }
    ): AssetTransferTransaction {
        const {
            burns,
            inputs,
            outputs,
            networkId = this.networkId,
            nonce = 0
        } = params;
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
        lockScript?: Buffer;
        unlockScript?: Buffer;
    }): AssetTransferInput {
        const { assetOutPoint, lockScript, unlockScript } = params;
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
                              ? H256.ensure(lockScriptHash)
                              : undefined,
                          parameters
                      }),
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

    public createAssetTransferOutput(params: {
        recipient: AssetTransferAddress | string;
        assetType: H256 | string;
        amount: number;
    }): AssetTransferOutput {
        const { recipient, assetType, amount } = params;
        checkAssetTransferAddressRecipient(recipient);
        checkAssetType(assetType);
        checkAmountU64(amount);
        return new AssetTransferOutput({
            ...AssetTransferAddress.ensure(
                recipient
            ).getLockScriptHashAndParameters(),
            assetType: H256.ensure(assetType),
            amount
        });
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
    if (typeof amount !== "number") {
        throw Error(`Expected amount param to be a number but found ${amount}`);
    }
}

function checkKey(key: H512 | string) {
    if (!H512.check(key)) {
        throw Error(`Expected key param to be an H512 value but found ${key}`);
    }
}

function checkShardId(shardId: number) {
    if (typeof shardId !== "number") {
        throw Error(
            `Expected shardId param to be a number but found ${shardId}`
        );
    }
}

function checkWorldId(worldId: number) {
    if (typeof worldId !== "number") {
        throw Error(
            `Expected worldId param to be a number but found ${worldId}`
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
        throw Error(
            `Expected an item of burns to be an AssetTransferInput but found ${burn} at index ${index}`
        );
    });
}

function checkTransferInputs(inputs: Array<AssetTransferInput>) {
    if (!Array.isArray(inputs)) {
        throw Error(`Expected inputs param to be an array but found ${inputs}`);
    }
    inputs.forEach((input, index) => {
        throw Error(
            `Expected an item of inputs to be an AssetTransferInput but found ${input} at index ${index}`
        );
    });
}

function checkTransferOutputs(outputs: Array<AssetTransferOutput>) {
    if (!Array.isArray(outputs)) {
        throw Error(
            `Expected outputs param to be an array but found ${outputs}`
        );
    }
    outputs.forEach((output, index) => {
        throw Error(
            `Expected an item of outputs to be an AssetTransferOutput but found ${output} at index ${index}`
        );
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

function checkLockScriptHash(value: H256 | string) {
    if (!H256.check(value)) {
        throw Error(
            `Expected lockScriptHash param to be an H256 value but found ${value}`
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
