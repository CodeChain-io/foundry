"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const buffer_1 = require("buffer");
const lib_1 = require("codechain-primitives/lib");
const P2PKH_1 = require("../../key/P2PKH");
const P2PKHBurn_1 = require("../../key/P2PKHBurn");
const U256_1 = require("../U256");
class AssetMintOutput {
    /**
     * Create an AssetMintOutput from an AssetMintOutput JSON object.
     * @param data An AssetMintOutput JSON object.
     * @returns An AssetMintOutput.
     */
    static fromJSON(data) {
        const { lockScriptHash, parameters, amount } = data;
        return new this({
            lockScriptHash: lib_1.H160.ensure(lockScriptHash),
            parameters: parameters.map(p => buffer_1.Buffer.from(p)),
            amount: amount == null ? null : U256_1.U256.ensure(amount)
        });
    }
    /**
     * @param data.lockScriptHash A lock script hash of the output.
     * @param data.parameters Parameters of the output.
     * @param data.amount Asset amount of the output.
     */
    constructor(data) {
        if ("recipient" in data) {
            // FIXME: Clean up by abstracting the standard scripts
            const { type, payload } = data.recipient;
            if ("pubkeys" in payload) {
                throw Error(`Multisig payload is not supported yet`);
            }
            switch (type) {
                case 0x00: // LOCK_SCRIPT_HASH ONLY
                    this.lockScriptHash = payload;
                    this.parameters = [];
                    break;
                case 0x01: // P2PKH
                    this.lockScriptHash = P2PKH_1.P2PKH.getLockScriptHash();
                    this.parameters = [buffer_1.Buffer.from(payload.value, "hex")];
                    break;
                case 0x02: // P2PKHBurn
                    this.lockScriptHash = P2PKHBurn_1.P2PKHBurn.getLockScriptHash();
                    this.parameters = [buffer_1.Buffer.from(payload.value, "hex")];
                    break;
                default:
                    throw Error(`Unexpected type of AssetTransferAddress: ${type}, ${data.recipient}`);
            }
        }
        else {
            const { lockScriptHash, parameters } = data;
            this.lockScriptHash = lockScriptHash;
            this.parameters = parameters;
        }
        this.amount = data.amount;
    }
    /**
     * Convert to an AssetMintOutput JSON object.
     * @returns An AssetMintOutput JSON object.
     */
    toJSON() {
        return {
            lockScriptHash: this.lockScriptHash.value,
            parameters: this.parameters.map(p => [...p]),
            amount: this.amount == null ? undefined : this.amount.toEncodeObject()
        };
    }
}
exports.AssetMintOutput = AssetMintOutput;
