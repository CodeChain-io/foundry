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
const codechain_keystore_1 = require("codechain-keystore");
class LocalKeyStore {
    constructor(cckey) {
        this.platform = {
            getKeyList: () => {
                return this.cckey.platform.getKeys();
            },
            createKey: (params = {}) => {
                return this.cckey.platform.createKey(params);
            },
            removeKey: (params) => {
                const { key } = params;
                return this.cckey.platform.deleteKey({ key });
            },
            exportRawKey: (params) => {
                const { passphrase = "" } = params;
                return this.cckey.platform.exportRawKey(Object.assign({}, params, { passphrase }));
            },
            getPublicKey: (params) => {
                const { key, passphrase = "" } = params;
                return this.cckey.platform.getPublicKey({ key, passphrase });
            },
            sign: (params) => {
                const { passphrase = "" } = params;
                return this.cckey.platform.sign(Object.assign({}, params, { passphrase }));
            }
        };
        this.asset = {
            getKeyList: () => {
                return this.cckey.asset.getKeys();
            },
            createKey: (params = {}) => {
                return this.cckey.asset.createKey(params);
            },
            removeKey: (params) => {
                const { key } = params;
                return this.cckey.asset.deleteKey({ key });
            },
            exportRawKey: (params) => {
                const { passphrase = "" } = params;
                return this.cckey.asset.exportRawKey(Object.assign({}, params, { passphrase }));
            },
            getPublicKey: (params) => {
                const { key, passphrase = "" } = params;
                return this.cckey.asset.getPublicKey({ key, passphrase });
            },
            sign: (params) => {
                const { passphrase = "" } = params;
                return this.cckey.asset.sign(Object.assign({}, params, { passphrase }));
            }
        };
        this.cckey = cckey;
    }
    static create(options = {}) {
        return __awaiter(this, void 0, void 0, function* () {
            const cckey = yield codechain_keystore_1.CCKey.create(options);
            return new LocalKeyStore(cckey);
        });
    }
    static createForTest() {
        return __awaiter(this, void 0, void 0, function* () {
            const cckey = yield codechain_keystore_1.CCKey.create({ dbType: "in-memory" });
            return new LocalKeyStore(cckey);
        });
    }
    close() {
        return this.cckey.close();
    }
}
exports.LocalKeyStore = LocalKeyStore;
