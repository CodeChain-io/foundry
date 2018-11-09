"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const buffer_1 = require("buffer");
const codechain_primitives_1 = require("codechain-primitives");
const P2PKH_1 = require("../../key/P2PKH");
const P2PKHBurn_1 = require("../../key/P2PKHBurn");
const H256_1 = require("../H256");
const U256_1 = require("../U256");
/**
 * An AssetTransferOutput consists of:
 *  - A lock script hash and parameters, which mark ownership of the asset.
 *  - An asset type and amount, which indicate the asset's type and quantity.
 */
class AssetTransferOutput {
    /**
     * Create an AssetTransferOutput from an AssetTransferOutput JSON object.
     * @param data An AssetTransferOutput JSON object.
     * @returns An AssetTransferOutput.
     */
    static fromJSON(data) {
        const { lockScriptHash, parameters, assetType, amount } = data;
        return new this({
            lockScriptHash: codechain_primitives_1.H160.ensure(lockScriptHash),
            parameters: parameters.map((p) => buffer_1.Buffer.from(p)),
            assetType: H256_1.H256.ensure(assetType),
            amount: U256_1.U256.ensure(amount)
        });
    }
    /**
     * @param data.lockScriptHash A lock script hash of the output.
     * @param data.parameters Parameters of the output.
     * @param data.assetType An asset type of the output.
     * @param data.amount An asset amount of the output.
     */
    constructor(data) {
        if ("recipient" in data) {
            // FIXME: Clean up by abstracting the standard scripts
            const { type, payload } = data.recipient;
            if ("pubkeys" in payload) {
                throw Error("Multisig payload is not supported yet");
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
        const { assetType, amount } = data;
        this.assetType = assetType;
        this.amount = amount;
    }
    /**
     * Convert to an object for RLP encoding.
     */
    toEncodeObject() {
        const { lockScriptHash, parameters, assetType, amount } = this;
        return [
            lockScriptHash.toEncodeObject(),
            parameters.map(parameter => buffer_1.Buffer.from(parameter)),
            assetType.toEncodeObject(),
            amount.toEncodeObject()
        ];
    }
    /**
     * Convert to an AssetTransferOutput JSON object.
     * @returns An AssetTransferOutput JSON object.
     */
    toJSON() {
        const { lockScriptHash, parameters, assetType, amount } = this;
        return {
            lockScriptHash: lockScriptHash.value,
            parameters: parameters.map(parameter => [...parameter]),
            assetType: assetType.value,
            amount: amount.toEncodeObject()
        };
    }
    /**
     * Get the shard ID.
     * @returns A shard ID.
     */
    shardId() {
        const { assetType } = this;
        return parseInt(assetType.value.slice(4, 8), 16);
    }
}
exports.AssetTransferOutput = AssetTransferOutput;
