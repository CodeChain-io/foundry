import {
    AssetTransferAddress,
    H256,
    PlatformAddress
} from "codechain-primitives";

import {
    AssetTransferInput,
    Order,
    SignedTransaction,
    Transaction,
    U64,
    UnwrapCCC
} from "../core/classes";
import { AssetTransaction } from "../core/Transaction";
import { ComposeAsset } from "../core/transaction/ComposeAsset";
import { DecomposeAsset } from "../core/transaction/DecomposeAsset";
import { TransferAsset } from "../core/transaction/TransferAsset";
import { NetworkId } from "../core/types";
import { SignatureTag } from "../utils";

import { KeyStore } from "./KeyStore";
import { LocalKeyStore } from "./LocalKeyStore";
import { MemoryKeyStore } from "./MemoryKeyStore";
import { P2PKH } from "./P2PKH";
import { P2PKHBurn } from "./P2PKHBurn";
import { RemoteKeyStore } from "./RemoteKeyStore";

export type KeyStoreType =
    | "local"
    | "memory"
    | { type: "remote"; url: string }
    | { type: "local"; path: string };

export class Key {
    public static classes = {
        RemoteKeyStore,
        LocalKeyStore
    };

    public classes = Key.classes;
    private networkId: NetworkId;
    private keyStore: KeyStore | null;
    private keyStoreType: KeyStoreType;

    constructor(options: { networkId: NetworkId; keyStoreType: KeyStoreType }) {
        const { networkId, keyStoreType } = options;
        if (!isKeyStoreType(keyStoreType)) {
            throw Error(`Unexpected keyStoreType param: ${keyStoreType}`);
        }
        this.networkId = networkId;
        this.keyStore = null;
        this.keyStoreType = keyStoreType;
    }

    /**
     * Creates persistent key store
     * @param keystoreURL key store url (ex http://localhost:7007)
     */
    public createRemoteKeyStore(keystoreURL: string): Promise<KeyStore> {
        return RemoteKeyStore.create(keystoreURL);
    }

    /**
     * Creates persistent key store which stores data in the filesystem.
     * @param dbPath A keystore file path
     */
    public createLocalKeyStore(dbPath?: string): Promise<KeyStore> {
        return LocalKeyStore.create({ dbPath });
    }

    /**
     * Creates a new platform address
     * @param params.keyStore A key store.
     * @returns A new platform address
     */
    public async createPlatformAddress(
        params: {
            keyStore?: KeyStore;
            passphrase?: string;
        } = {}
    ): Promise<PlatformAddress> {
        const { keyStore = await this.ensureKeyStore(), passphrase } = params;
        if (!isKeyStore(keyStore)) {
            throw Error(
                `Expected keyStore param to be a KeyStore instance but found ${keyStore}`
            );
        }
        const accountId = await keyStore.platform.createKey({ passphrase });
        const { networkId } = this;
        return PlatformAddress.fromAccountId(accountId, { networkId });
    }

    /**
     * Creates a new asset transfer address
     * @param params.type The type of AssetTransferAddress. The default value is "P2PKH".
     * @param params.keyStore A key store.
     * @returns A new platform address
     */
    public async createAssetTransferAddress(
        params: {
            type?: "P2PKH" | "P2PKHBurn";
            keyStore?: KeyStore;
            passphrase?: string;
        } = {}
    ): Promise<AssetTransferAddress> {
        const {
            keyStore = await this.ensureKeyStore(),
            type = "P2PKH",
            passphrase
        } = params;
        if (!isKeyStore(keyStore)) {
            throw Error(
                `Expected keyStore param to be a KeyStore instance but found ${keyStore}`
            );
        }
        const { networkId } = this;
        if (type === "P2PKH") {
            const p2pkh = new P2PKH({ keyStore, networkId });
            return p2pkh.createAddress({ passphrase });
        } else if (type === "P2PKHBurn") {
            const p2pkhBurn = new P2PKHBurn({ keyStore, networkId });
            return p2pkhBurn.createAddress({ passphrase });
        } else {
            throw Error(
                `Expected the type param of createAssetTransferAddress to be either P2PKH or P2PKHBurn but found ${type}`
            );
        }
    }

    /**
     * Creates P2PKH script generator.
     * @returns new instance of P2PKH
     */
    public createP2PKH(params: { keyStore: KeyStore }): P2PKH {
        const { keyStore } = params;
        if (!isKeyStore(keyStore)) {
            throw Error(
                `Expected keyStore param to be a KeyStore instance but found ${keyStore}`
            );
        }
        const { networkId } = this;
        return new P2PKH({ keyStore, networkId });
    }

    /**
     * Creates P2PKHBurn script generator.
     * @returns new instance of P2PKHBurn
     */
    public createP2PKHBurn(params: { keyStore: KeyStore }): P2PKHBurn {
        const { keyStore } = params;
        if (!isKeyStore(keyStore)) {
            throw Error(
                `Expected keyStore param to be a KeyStore instance but found ${keyStore}`
            );
        }
        const { networkId } = this;
        return new P2PKHBurn({ keyStore, networkId });
    }

    /**
     * Approves the transaction
     * @param transaction A transaction
     * @param params
     * @param params.keyStore A key store.
     * @param params.account An account.
     * @param params.passphrase The passphrase for the given account
     * @returns An approval
     */
    public async approveTransaction(
        transaction: AssetTransaction,
        params: {
            keyStore?: KeyStore;
            account: PlatformAddress | string;
            passphrase?: string;
        }
    ): Promise<string> {
        const {
            account,
            passphrase,
            keyStore = await this.ensureKeyStore()
        } = params;
        if (!isKeyStore(keyStore)) {
            throw Error(
                `Expected keyStore param to be a KeyStore instance but found ${keyStore}`
            );
        }
        if (!PlatformAddress.check(account)) {
            throw Error(
                `Expected account param to be a PlatformAddress value but found ${account}`
            );
        }
        const accountId = PlatformAddress.ensure(account).getAccountId();
        return await keyStore.platform.sign({
            key: accountId.value,
            message: transaction.tracker().value,
            passphrase
        });
    }
    /**
     * Signs a Transaction with the given account.
     * @param tx A Transaction
     * @param params.keyStore A key store.
     * @param params.account An account.
     * @param params.passphrase The passphrase for the given account
     * @returns A SignedTransaction
     * @throws When seq or fee in the Transaction is null
     * @throws When account or its passphrase is invalid
     */
    public async signTransaction(
        tx: Transaction,
        params: {
            keyStore?: KeyStore;
            account: PlatformAddress | string;
            passphrase?: string;
            fee: U64 | string | number;
            seq: number;
        }
    ): Promise<SignedTransaction> {
        if (!(tx instanceof Transaction)) {
            throw Error(
                `Expected the first argument of signTransaction to be a Transaction instance but found ${tx}`
            );
        }
        const {
            account,
            passphrase,
            keyStore = await this.ensureKeyStore(),
            fee,
            seq
        } = params;
        if (!isKeyStore(keyStore)) {
            throw Error(
                `Expected keyStore param to be a KeyStore instance but found ${keyStore}`
            );
        }
        if (!PlatformAddress.check(account)) {
            throw Error(
                `Expected account param to be a PlatformAddress value but found ${account}`
            );
        }
        if (!U64.check(fee)) {
            throw Error(
                `Expected fee param to be a U64 value but found ${fee}`
            );
        }
        if (typeof seq !== "number") {
            throw Error(
                `Expected seq param to be a number value but found ${seq}`
            );
        }
        tx.setFee(fee);
        tx.setSeq(seq);
        const accountId = PlatformAddress.ensure(account).getAccountId();
        const sig = await keyStore.platform.sign({
            key: accountId.value,
            message: tx.unsignedHash().value,
            passphrase
        });
        return new SignedTransaction(tx, sig);
    }

    /**
     * Signs a transaction's input with a given key store.
     * @param tx An TransferAsset.
     * @param index The index of an input to sign.
     * @param params.keyStore A key store.
     * @param params.passphrase The passphrase for the given input.
     */
    public async signTransactionInput(
        tx: TransferAsset | ComposeAsset | DecomposeAsset,
        index: number,
        params: {
            keyStore?: KeyStore;
            passphrase?: string;
            signatureTag?: SignatureTag;
        } = {}
    ): Promise<void> {
        const input = tx.input(index);
        if (input == null) {
            throw Error(`Invalid index`);
        }
        const { lockScriptHash, parameters } = input.prevOut;
        if (lockScriptHash === undefined || parameters === undefined) {
            throw Error(`Invalid transaction input`);
        }
        if (lockScriptHash.value !== P2PKH.getLockScriptHash().value) {
            throw Error(`Unexpected lock script hash`);
        }
        if (parameters.length !== 1) {
            throw Error(`Unexpected length of parameters`);
        }
        const publicKeyHash = Buffer.from(parameters[0]).toString("hex");

        input.setLockScript(P2PKH.getLockScript());
        const {
            keyStore = await this.ensureKeyStore(),
            passphrase,
            signatureTag = { input: "all", output: "all" } as SignatureTag
        } = params;
        const p2pkh = this.createP2PKH({ keyStore });
        let message: H256;
        if (tx instanceof TransferAsset) {
            let flag = false;
            for (const order of tx.orders()) {
                if (order.inputIndices.indexOf(index) !== -1) {
                    message = order.order.hash();
                    flag = true;
                    break;
                }
            }
            if (!flag) {
                message = tx.hashWithoutScript({
                    tag: signatureTag,
                    type: "input",
                    index
                });
            }
        } else if (tx instanceof ComposeAsset) {
            // FIXME: check type
            message = tx.hashWithoutScript({
                tag: signatureTag,
                index
            });
        } else if (tx instanceof DecomposeAsset) {
            // FIXME: check signature tag
            message = tx.hashWithoutScript();
        } else {
            throw Error(`Invalid tx`);
        }
        input.setUnlockScript(
            await p2pkh.createUnlockScript(publicKeyHash, message!, {
                passphrase,
                signatureTag
            })
        );
    }

    /**
     * Signs a transaction's input with an order.
     * @param input An AssetTransferInput.
     * @param order An order to be used as a signature message.
     * @param params.keyStore A key store.
     * @param params.passphrase The passphrase for the given input.
     */
    public async signTransactionInputWithOrder(
        input: AssetTransferInput,
        order: Order,
        params: {
            keyStore?: KeyStore;
            passphrase?: string;
        } = {}
    ): Promise<void> {
        const { lockScriptHash, parameters } = input.prevOut;
        if (lockScriptHash === undefined || parameters === undefined) {
            throw Error(`Invalid transaction input`);
        }
        if (lockScriptHash.value !== P2PKH.getLockScriptHash().value) {
            throw Error(`Unexpected lock script hash`);
        }
        if (parameters.length !== 1) {
            throw Error(`Unexpected length of parameters`);
        }
        const publicKeyHash = Buffer.from(parameters[0]).toString("hex");

        input.setLockScript(P2PKH.getLockScript());
        const { keyStore = await this.ensureKeyStore(), passphrase } = params;
        const p2pkh = this.createP2PKH({ keyStore });
        input.setUnlockScript(
            await p2pkh.createUnlockScript(publicKeyHash, order.hash(), {
                passphrase,
                signatureTag: { input: "all", output: "all" }
            })
        );
    }

    /**
     * Signs a transaction's burn with a given key store.
     * @param tx An TransferAsset.
     * @param index The index of a burn to sign.
     * @param params.keyStore A key store.
     * @param params.passphrase The passphrase for the given burn.
     */
    public async signTransactionBurn(
        tx: TransferAsset | UnwrapCCC,
        index: number,
        params: {
            keyStore?: KeyStore;
            passphrase?: string;
            signatureTag?: SignatureTag;
        } = {}
    ): Promise<void> {
        const burn = tx.burn(index);
        if (burn == null) {
            throw Error(`Invalid index`);
        }
        const { lockScriptHash, parameters } = burn.prevOut;
        if (lockScriptHash === undefined || parameters === undefined) {
            throw Error(`Invalid transaction burn`);
        }
        if (lockScriptHash.value !== P2PKHBurn.getLockScriptHash().value) {
            throw Error(`Unexpected lock script hash`);
        }
        if (parameters.length !== 1) {
            throw Error(`Unexpected length of parameters`);
        }
        const publicKeyHash = Buffer.from(parameters[0]).toString("hex");

        burn.setLockScript(P2PKHBurn.getLockScript());
        const {
            keyStore = await this.ensureKeyStore(),
            passphrase,
            signatureTag = { input: "all", output: "all" } as SignatureTag
        } = params;
        const p2pkhBurn = this.createP2PKHBurn({ keyStore });
        burn.setUnlockScript(
            await p2pkhBurn.createUnlockScript(
                publicKeyHash,
                tx.hashWithoutScript({
                    tag: signatureTag,
                    type: "burn",
                    index
                }),
                {
                    passphrase,
                    signatureTag
                }
            )
        );
    }

    private async ensureKeyStore(): Promise<KeyStore> {
        if (this.keyStore == null) {
            if (this.keyStoreType === "local") {
                this.keyStore = await LocalKeyStore.create();
            } else if (this.keyStoreType === "memory") {
                this.keyStore = await LocalKeyStore.createForTest();
            } else if (this.keyStoreType.type === "local") {
                this.keyStore = await LocalKeyStore.create({
                    dbPath: this.keyStoreType.path
                });
            } else if (this.keyStoreType.type === "remote") {
                this.keyStore = await RemoteKeyStore.create(
                    this.keyStoreType.url
                );
            } else {
                throw Error(`Unreachable`);
            }
        }
        return this.keyStore;
    }
}

function isKeyStore(value: any) {
    return (
        value instanceof LocalKeyStore ||
        value instanceof RemoteKeyStore ||
        value instanceof MemoryKeyStore
    );
}

function isKeyStoreType(value: any) {
    if (typeof value === "string") {
        return value === "local" || value === "memory";
    }
    if (typeof value === "object" && value != null) {
        return (
            (value.type === "local" && typeof value.path === "string") ||
            (value.type === "remote" && typeof value.url === "string")
        );
    }
    return false;
}
