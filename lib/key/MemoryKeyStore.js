"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const _ = require("lodash");
const H256_1 = require("../core/H256");
const utils_1 = require("../utils");
/**
 * @hidden
 */
class KeyManager {
    constructor(keyMaker) {
        this.privateKeyMap = {};
        this.passphraseMap = {};
        this.publicKeyMap = {};
        this.mappingKeyMaker = keyMaker;
    }
    getKeyList() {
        return Promise.resolve(_.keys(this.privateKeyMap));
    }
    createKey(params = {}) {
        const privateKey = utils_1.generatePrivateKey();
        const publicKey = utils_1.getPublicFromPrivate(privateKey);
        const key = this.mappingKeyMaker(publicKey);
        this.privateKeyMap[key] = privateKey;
        this.passphraseMap[key] = params.passphrase || "";
        this.publicKeyMap[key] = publicKey;
        return Promise.resolve(key);
    }
    removeKey(params) {
        const { key } = params;
        if (this.privateKeyMap[key]) {
            delete this.privateKeyMap[key];
            delete this.publicKeyMap[key];
            delete this.passphraseMap[key];
            return Promise.resolve(true);
        }
        else {
            return Promise.resolve(false);
        }
    }
    exportRawKey(params) {
        const { passphrase = "", key } = params;
        if (passphrase !== this.passphraseMap[key]) {
            return Promise.reject("The passphrase does not match");
        }
        return Promise.resolve(this.privateKeyMap[key]);
    }
    getPublicKey(params) {
        const { key } = params;
        if (this.publicKeyMap[key]) {
            return Promise.resolve(this.publicKeyMap[key]);
        }
        else {
            return Promise.resolve(null);
        }
    }
    sign(params) {
        const { key, message, passphrase = "" } = params;
        if (passphrase !== this.passphraseMap[key]) {
            return Promise.reject("The passphrase does not match");
        }
        const { r, s, v } = utils_1.signEcdsa(message, this.privateKeyMap[key]);
        const sig = `${_.padStart(r, 64, "0")}${_.padStart(s, 64, "0")}${_.padStart(v.toString(16), 2, "0")}`;
        return Promise.resolve(sig);
    }
}
class MemoryKeyStore {
    constructor() {
        this.platform = new KeyManager(utils_1.getAccountIdFromPublic);
        this.asset = new KeyManager(this.getHash);
    }
    getHash(publicKey) {
        return H256_1.H256.ensure(utils_1.blake256(publicKey)).value;
    }
}
exports.MemoryKeyStore = MemoryKeyStore;
