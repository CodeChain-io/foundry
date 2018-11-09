"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : new P(function (resolve) { resolve(result.value); }).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
const codechain_primitives_1 = require("codechain-primitives");
const classes_1 = require("../core/classes");
const LocalKeyStore_1 = require("./LocalKeyStore");
const MemoryKeyStore_1 = require("./MemoryKeyStore");
const P2PKH_1 = require("./P2PKH");
const P2PKHBurn_1 = require("./P2PKHBurn");
const RemoteKeyStore_1 = require("./RemoteKeyStore");
class Key {
    constructor(options) {
        this.classes = Key.classes;
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
    createRemoteKeyStore(keystoreURL) {
        return RemoteKeyStore_1.RemoteKeyStore.create(keystoreURL);
    }
    /**
     * Creates persistent key store which stores data in the filesystem.
     * @param dbPath A keystore file path
     */
    createLocalKeyStore(dbPath) {
        return LocalKeyStore_1.LocalKeyStore.create({ dbPath });
    }
    /**
     * Creates a new platform address
     * @param params.keyStore A key store.
     * @returns A new platform address
     */
    createPlatformAddress(params = {}) {
        return __awaiter(this, void 0, void 0, function* () {
            const { keyStore = yield this.ensureKeyStore(), passphrase } = params;
            if (!isKeyStore(keyStore)) {
                throw Error(`Expected keyStore param to be a KeyStore instance but found ${keyStore}`);
            }
            const accountId = yield keyStore.platform.createKey({ passphrase });
            const { networkId } = this;
            return codechain_primitives_1.PlatformAddress.fromAccountId(accountId, { networkId });
        });
    }
    /**
     * Creates a new asset transfer address
     * @param params.type The type of AssetTransferAddress. The default value is "P2PKH".
     * @param params.keyStore A key store.
     * @returns A new platform address
     */
    createAssetTransferAddress(params = {}) {
        return __awaiter(this, void 0, void 0, function* () {
            const { keyStore = yield this.ensureKeyStore(), type = "P2PKH", passphrase } = params;
            if (!isKeyStore(keyStore)) {
                throw Error(`Expected keyStore param to be a KeyStore instance but found ${keyStore}`);
            }
            const { networkId } = this;
            if (type === "P2PKH") {
                const p2pkh = new P2PKH_1.P2PKH({ keyStore, networkId });
                return p2pkh.createAddress({ passphrase });
            }
            else if (type === "P2PKHBurn") {
                const p2pkhBurn = new P2PKHBurn_1.P2PKHBurn({ keyStore, networkId });
                return p2pkhBurn.createAddress({ passphrase });
            }
            else {
                throw Error(`Expected the type param of createAssetTransferAddress to be either P2PKH or P2PKHBurn but found ${type}`);
            }
        });
    }
    /**
     * Creates P2PKH script generator.
     * @returns new instance of P2PKH
     */
    createP2PKH(params) {
        const { keyStore } = params;
        if (!isKeyStore(keyStore)) {
            throw Error(`Expected keyStore param to be a KeyStore instance but found ${keyStore}`);
        }
        const { networkId } = this;
        return new P2PKH_1.P2PKH({ keyStore, networkId });
    }
    /**
     * Creates P2PKHBurn script generator.
     * @returns new instance of P2PKHBurn
     */
    createP2PKHBurn(params) {
        const { keyStore } = params;
        if (!isKeyStore(keyStore)) {
            throw Error(`Expected keyStore param to be a KeyStore instance but found ${keyStore}`);
        }
        const { networkId } = this;
        return new P2PKHBurn_1.P2PKHBurn({ keyStore, networkId });
    }
    /**
     * Signs a Parcel with the given account.
     * @param parcel A Parcel
     * @param params.keyStore A key store.
     * @param params.account An account.
     * @param params.passphrase The passphrase for the given account
     * @returns A SignedParcel
     * @throws When seq or fee in the Parcel is null
     * @throws When account or its passphrase is invalid
     */
    signParcel(parcel, params) {
        return __awaiter(this, void 0, void 0, function* () {
            if (!(parcel instanceof classes_1.Parcel)) {
                throw Error(`Expected the first argument of signParcel to be a Parcel instance but found ${parcel}`);
            }
            const { account, passphrase, keyStore = yield this.ensureKeyStore(), fee, seq } = params;
            if (!isKeyStore(keyStore)) {
                throw Error(`Expected keyStore param to be a KeyStore instance but found ${keyStore}`);
            }
            if (!codechain_primitives_1.PlatformAddress.check(account)) {
                throw Error(`Expected account param to be a PlatformAddress value but found ${account}`);
            }
            if (!classes_1.U256.check(fee)) {
                throw Error(`Expected fee param to be a U256 value but found ${fee}`);
            }
            if (!classes_1.U256.check(seq)) {
                throw Error(`Expected seq param to be a U256 value but found ${seq}`);
            }
            parcel.setFee(fee);
            parcel.setSeq(seq);
            const accountId = codechain_primitives_1.PlatformAddress.ensure(account).getAccountId();
            const sig = yield keyStore.platform.sign({
                key: accountId.value,
                message: parcel.hash().value,
                passphrase
            });
            return new classes_1.SignedParcel(parcel, sig);
        });
    }
    /**
     * Signs a transaction's input with a given key store.
     * @param tx An AssetTransferTransaction.
     * @param index The index of an input to sign.
     * @param params.keyStore A key store.
     * @param params.passphrase The passphrase for the given input.
     */
    signTransactionInput(tx, index, params = {}) {
        return __awaiter(this, void 0, void 0, function* () {
            if ("inputs" in tx && index >= tx.inputs.length) {
                throw Error(`Invalid index`);
            }
            if ("input" in tx && index >= 1) {
                throw Error(`Invalid index`);
            }
            const input = "inputs" in tx ? tx.inputs[index] : tx.input;
            const { lockScriptHash, parameters } = input.prevOut;
            if (lockScriptHash === undefined || parameters === undefined) {
                throw Error(`Invalid transaction input`);
            }
            if (lockScriptHash.value !== P2PKH_1.P2PKH.getLockScriptHash().value) {
                throw Error(`Unexpected lock script hash`);
            }
            if (parameters.length !== 1) {
                throw Error(`Unexpected length of parameters`);
            }
            const publicKeyHash = Buffer.from(parameters[0]).toString("hex");
            input.setLockScript(P2PKH_1.P2PKH.getLockScript());
            const { keyStore = yield this.ensureKeyStore(), passphrase, signatureTag = { input: "all", output: "all" } } = params;
            const p2pkh = this.createP2PKH({ keyStore });
            let message;
            if (tx instanceof classes_1.AssetTransferTransaction) {
                message = tx.hashWithoutScript({
                    tag: signatureTag,
                    type: "input",
                    index
                });
            }
            else if (tx instanceof classes_1.AssetComposeTransaction) {
                // FIXME: check type
                message = tx.hashWithoutScript({
                    tag: signatureTag,
                    index
                });
            }
            else if (tx instanceof classes_1.AssetDecomposeTransaction) {
                // FIXME: check signature tag
                message = tx.hashWithoutScript();
            }
            else {
                throw Error(`Invalid tx`);
            }
            input.setUnlockScript(yield p2pkh.createUnlockScript(publicKeyHash, message, {
                passphrase,
                signatureTag
            }));
        });
    }
    /**
     * Signs a transaction's burn with a given key store.
     * @param tx An AssetTransferTransaction.
     * @param index The index of a burn to sign.
     * @param params.keyStore A key store.
     * @param params.passphrase The passphrase for the given burn.
     */
    signTransactionBurn(tx, index, params = {}) {
        return __awaiter(this, void 0, void 0, function* () {
            if (index >= tx.burns.length) {
                throw Error(`Invalid index`);
            }
            const { lockScriptHash, parameters } = tx.burns[index].prevOut;
            if (lockScriptHash === undefined || parameters === undefined) {
                throw Error(`Invalid transaction burn`);
            }
            if (lockScriptHash.value !== P2PKHBurn_1.P2PKHBurn.getLockScriptHash().value) {
                throw Error(`Unexpected lock script hash`);
            }
            if (parameters.length !== 1) {
                throw Error(`Unexpected length of parameters`);
            }
            const publicKeyHash = Buffer.from(parameters[0]).toString("hex");
            tx.burns[index].setLockScript(P2PKHBurn_1.P2PKHBurn.getLockScript());
            const { keyStore = yield this.ensureKeyStore(), passphrase, signatureTag = { input: "all", output: "all" } } = params;
            const p2pkhBurn = this.createP2PKHBurn({ keyStore });
            tx.burns[index].setUnlockScript(yield p2pkhBurn.createUnlockScript(publicKeyHash, tx.hashWithoutScript({
                tag: signatureTag,
                type: "burn",
                index
            }), {
                passphrase,
                signatureTag
            }));
        });
    }
    ensureKeyStore() {
        return __awaiter(this, void 0, void 0, function* () {
            if (this.keyStore === null) {
                if (this.keyStoreType === "local") {
                    this.keyStore = yield LocalKeyStore_1.LocalKeyStore.create();
                }
                else if (this.keyStoreType === "memory") {
                    this.keyStore = yield LocalKeyStore_1.LocalKeyStore.createForTest();
                }
                else if (this.keyStoreType.type === "local") {
                    this.keyStore = yield LocalKeyStore_1.LocalKeyStore.create({
                        dbPath: this.keyStoreType.path
                    });
                }
                else if (this.keyStoreType.type === "remote") {
                    this.keyStore = yield RemoteKeyStore_1.RemoteKeyStore.create(this.keyStoreType.url);
                }
                else {
                    throw Error(`Unreachable`);
                }
            }
            return this.keyStore;
        });
    }
}
Key.classes = {
    RemoteKeyStore: RemoteKeyStore_1.RemoteKeyStore,
    LocalKeyStore: LocalKeyStore_1.LocalKeyStore
};
exports.Key = Key;
function isKeyStore(value) {
    return (value instanceof LocalKeyStore_1.LocalKeyStore ||
        value instanceof RemoteKeyStore_1.RemoteKeyStore ||
        value instanceof MemoryKeyStore_1.MemoryKeyStore);
}
function isKeyStoreType(value) {
    if (typeof value === "string") {
        return value === "local" || value === "memory";
    }
    if (typeof value === "object" && value !== null) {
        return ((value.type === "local" && typeof value.path === "string") ||
            (value.type === "remote" && typeof value.url === "string"));
    }
    return false;
}
