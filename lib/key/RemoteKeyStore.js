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
const rp = require("request-promise");
class RemoteKeyManager {
    constructor(keystoreURL, keyType) {
        this.keystoreURL = keystoreURL;
        this.keyType = keyType;
    }
    getKeyList() {
        return __awaiter(this, void 0, void 0, function* () {
            const response = yield rp.get(`${this.keystoreURL}/api/keys`, {
                body: { keyType: this.keyType },
                json: true
            });
            if (!response.success) {
                throw Error(response.error);
            }
            return response.result;
        });
    }
    createKey(params = {}) {
        return __awaiter(this, void 0, void 0, function* () {
            const response = yield rp.post(`${this.keystoreURL}/api/keys`, {
                body: {
                    keyType: this.keyType,
                    passphrase: params.passphrase
                },
                json: true
            });
            if (!response.success) {
                throw Error(response.error);
            }
            return response.result;
        });
    }
    removeKey(params) {
        return __awaiter(this, void 0, void 0, function* () {
            const response = yield rp.delete(`${this.keystoreURL}/api/keys/${params.key}`, {
                body: {
                    keyType: this.keyType
                },
                json: true
            });
            if (!response.success) {
                throw Error(response.error);
            }
            return response.result;
        });
    }
    exportRawKey(params) {
        return __awaiter(this, void 0, void 0, function* () {
            const response = yield rp.post(`${this.keystoreURL}/api/keys/${params.key}/sign`, {
                body: {
                    keyType: this.keyType,
                    passphrase: params.passphrase
                },
                json: true
            });
            if (!response.success) {
                throw Error(response.error);
            }
            return response.result;
        });
    }
    getPublicKey(params) {
        return __awaiter(this, void 0, void 0, function* () {
            const response = yield rp.get(`${this.keystoreURL}/api/keys/${params.key}/publicKey`, {
                body: {
                    keyType: this.keyType
                },
                json: true
            });
            if (!response.success) {
                throw Error(response.error);
            }
            return response.result;
        });
    }
    sign(params) {
        return __awaiter(this, void 0, void 0, function* () {
            const response = yield rp.post(`${this.keystoreURL}/api/keys/${params.key}/sign`, {
                body: {
                    keyType: this.keyType,
                    passphrase: params.passphrase,
                    publicKey: params.key,
                    message: params.message
                },
                json: true
            });
            if (!response.success) {
                throw Error(response.error);
            }
            return response.result;
        });
    }
}
class RemoteKeyStore {
    constructor(keystoreURL) {
        this.mapping = {
            get: (params) => __awaiter(this, void 0, void 0, function* () {
                const response = yield rp.get(`${this.keystoreURL}/api/mapping/${params.key}`, {
                    json: true
                });
                if (!response.success) {
                    throw Error(response.error);
                }
                return response.result;
            })
        };
        this.keystoreURL = keystoreURL;
        this.platform = new RemoteKeyManager(keystoreURL, "platform");
        this.asset = new RemoteKeyManager(keystoreURL, "asset");
    }
    static create(keystoreURL) {
        return __awaiter(this, void 0, void 0, function* () {
            const keystore = new RemoteKeyStore(keystoreURL);
            yield keystore.ping();
            return keystore;
        });
    }
    // Use only this method for test purpose
    static createUnsafe(keystoreURL) {
        const keystore = new RemoteKeyStore(keystoreURL);
        keystore.ping();
        return keystore;
    }
    ping() {
        return __awaiter(this, void 0, void 0, function* () {
            const response = yield rp.get(`${this.keystoreURL}/ping`, {
                json: true
            });
            if (!response.success) {
                throw Error(response.error);
            }
            return response.result;
        });
    }
}
exports.RemoteKeyStore = RemoteKeyStore;
