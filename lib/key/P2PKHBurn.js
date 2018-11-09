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
const buffer_1 = require("buffer");
const codechain_primitives_1 = require("codechain-primitives");
const Script_1 = require("../core/Script");
const utils_1 = require("../utils");
class P2PKHBurn {
    static getLockScript() {
        const { COPY, BLAKE160, EQ, JZ, CHKSIG, BURN } = Script_1.Script.Opcode;
        return buffer_1.Buffer.from([
            COPY,
            0x01,
            BLAKE160,
            EQ,
            JZ,
            0xff,
            CHKSIG,
            JZ,
            0xff,
            BURN
        ]);
    }
    static getLockScriptHash() {
        return new codechain_primitives_1.H160("37572bdcc22d39a59c0d12d301f6271ba3fdd451");
    }
    constructor(params) {
        const { keyStore, networkId } = params;
        this.keyStore = keyStore;
        this.networkId = networkId;
    }
    createAddress(options = {}) {
        return __awaiter(this, void 0, void 0, function* () {
            const { passphrase } = options;
            const hash = yield this.keyStore.asset.createKey({ passphrase });
            return codechain_primitives_1.AssetTransferAddress.fromTypeAndPayload(2, hash, {
                networkId: this.networkId
            });
        });
    }
    createUnlockScript(publicKeyHash, txhash, options = {}) {
        return __awaiter(this, void 0, void 0, function* () {
            const { passphrase, signatureTag = { input: "all", output: "all" } } = options;
            const publicKey = yield this.keyStore.asset.getPublicKey({
                key: publicKeyHash,
                passphrase
            });
            if (!publicKey) {
                throw Error(`Unable to get original key from the given public key hash: ${publicKeyHash}`);
            }
            const signature = yield this.keyStore.asset.sign({
                key: publicKeyHash,
                message: txhash.value,
                passphrase
            });
            const encodedTag = utils_1.encodeSignatureTag(signatureTag);
            const { PUSHB } = Script_1.Script.Opcode;
            return buffer_1.Buffer.from([
                PUSHB,
                65,
                ...buffer_1.Buffer.from(signature, "hex"),
                PUSHB,
                encodedTag.byteLength,
                ...encodedTag,
                PUSHB,
                64,
                ...buffer_1.Buffer.from(publicKey, "hex")
            ]);
        });
    }
}
exports.P2PKHBurn = P2PKHBurn;
