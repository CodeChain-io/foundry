import { Address, AddressValue, H256, U64Value } from "foundry-primitives";

import { SignedTransaction, Transaction, U64 } from "../core/classes";
import { AssetTransaction } from "../core/Transaction";
import { NetworkId } from "../core/types";
import { SignatureTag } from "../utils";

import { KeyStore } from "./KeyStore";
import { LocalKeyStore } from "./LocalKeyStore";
import { MemoryKeyStore } from "./MemoryKeyStore";
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
    public async createaddress(
        params: {
            keyStore?: KeyStore;
            passphrase?: string;
        } = {}
    ): Promise<Address> {
        const { keyStore = await this.ensureKeyStore(), passphrase } = params;
        if (!isKeyStore(keyStore)) {
            throw Error(
                `Expected keyStore param to be a KeyStore instance but found ${keyStore}`
            );
        }
        const accountId = await keyStore.platform.createKey({ passphrase });
        const { networkId } = this;
        return Address.fromAccountId(accountId, { networkId });
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
            account: AddressValue;
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
        if (!Address.check(account)) {
            throw Error(
                `Expected account param to be a address value but found ${account}`
            );
        }
        const accountId = Address.ensure(account).getAccountId();
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
            account: AddressValue;
            passphrase?: string;
            fee: U64Value;
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
        if (!Address.check(account)) {
            throw Error(
                `Expected account param to be a address value but found ${account}`
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
        const accountId = Address.ensure(account).getAccountId();
        const signerPublic = await keyStore.platform.getPublicKey({
            key: accountId.value,
            passphrase
        });
        if (signerPublic === null) {
            throw Error(
                `The account ${accountId.value} is not found in the Keystore`
            );
        }

        const sig = await keyStore.platform.sign({
            key: accountId.value,
            message: tx.unsignedHash().value,
            passphrase
        });
        return new SignedTransaction(tx, sig, signerPublic);
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
