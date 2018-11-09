"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const codechain_primitives_1 = require("codechain-primitives");
const _ = require("lodash");
const utils_1 = require("../../utils");
const Asset_1 = require("../Asset");
const H256_1 = require("../H256");
const U256_1 = require("../U256");
const AssetTransferInput_1 = require("./AssetTransferInput");
const AssetTransferOutput_1 = require("./AssetTransferOutput");
const RLP = require("rlp");
/**
 * Spends the existing asset and creates a new asset. Ownership can be transferred during this process.
 *
 * An AssetTransferTransaction consists of:
 *  - A list of AssetTransferInput to burn.
 *  - A list of AssetTransferInput to spend.
 *  - A list of AssetTransferOutput to create.
 *  - A network ID. This must be identical to the network ID of which the
 *  transaction is being sent to.
 *
 * All inputs must be valid for the transaction to be valid. When each asset
 * types' amount have been summed, the sum of inputs and the sum of outputs
 * must be identical.
 */
class AssetTransferTransaction {
    /**
     * @param params.burns An array of AssetTransferInput to burn.
     * @param params.inputs An array of AssetTransferInput to spend.
     * @param params.outputs An array of AssetTransferOutput to create.
     * @param params.networkId A network ID of the transaction.
     */
    constructor(params) {
        this.type = "assetTransfer";
        const { burns, inputs, outputs, networkId } = params;
        this.burns = burns;
        this.inputs = inputs;
        this.outputs = outputs;
        this.networkId = networkId;
    }
    /** Create an AssetTransferTransaction from an AssetTransferTransaction JSON object.
     * @param obj An AssetTransferTransaction JSON object.
     * @returns An AssetTransferTransaction.
     */
    static fromJSON(obj) {
        const { data: { networkId, burns, inputs, outputs } } = obj;
        return new this({
            burns: burns.map((input) => AssetTransferInput_1.AssetTransferInput.fromJSON(input)),
            inputs: inputs.map((input) => AssetTransferInput_1.AssetTransferInput.fromJSON(input)),
            outputs: outputs.map((output) => AssetTransferOutput_1.AssetTransferOutput.fromJSON(output)),
            networkId
        });
    }
    /**
     * Convert to an object for RLP encoding.
     */
    toEncodeObject() {
        return [
            4,
            this.networkId,
            this.burns.map(input => input.toEncodeObject()),
            this.inputs.map(input => input.toEncodeObject()),
            this.outputs.map(output => output.toEncodeObject())
        ];
    }
    /**
     * Convert to RLP bytes.
     */
    rlpBytes() {
        return RLP.encode(this.toEncodeObject());
    }
    /**
     * Get the hash of an AssetTransferTransaction.
     * @returns A transaction hash.
     */
    hash() {
        return new H256_1.H256(utils_1.blake256(this.rlpBytes()));
    }
    /**
     * Add an AssetTransferInput to burn.
     * @param burns An array of either an AssetTransferInput or an Asset.
     * @returns The AssetTransferTransaction, which is modified by adding them.
     */
    addBurns(burns, ...rest) {
        if (!Array.isArray(burns)) {
            burns = [burns, ...rest];
        }
        burns.forEach((burn, index) => {
            if (burn instanceof AssetTransferInput_1.AssetTransferInput) {
                this.burns.push(burn);
            }
            else if (burn instanceof Asset_1.Asset) {
                this.burns.push(burn.createTransferInput());
            }
            else {
                throw Error(`Expected burn param to be either AssetTransferInput or Asset but found ${burn} at ${index}`);
            }
        });
        return this;
    }
    /**
     * Add an AssetTransferInput to spend.
     * @param inputs An array of either an AssetTransferInput or an Asset.
     * @returns The AssetTransferTransaction, which is modified by adding them.
     */
    addInputs(inputs, ...rest) {
        if (!Array.isArray(inputs)) {
            inputs = [inputs, ...rest];
        }
        inputs.forEach((input, index) => {
            if (input instanceof AssetTransferInput_1.AssetTransferInput) {
                this.inputs.push(input);
            }
            else if (input instanceof Asset_1.Asset) {
                this.inputs.push(input.createTransferInput());
            }
            else {
                throw Error(`Expected input param to be either AssetTransferInput or Asset but found ${input} at ${index}`);
            }
        });
        return this;
    }
    /**
     * Add an AssetTransferOutput to create.
     * @param outputs An array of either an AssetTransferOutput or an object
     * that has amount, assetType and recipient values.
     * @param output.amount Asset amount of the output.
     * @param output.assetType An asset type of the output.
     * @param output.recipient A recipient of the output.
     */
    addOutputs(outputs, ...rest) {
        if (!Array.isArray(outputs)) {
            outputs = [outputs, ...rest];
        }
        outputs.forEach(output => {
            if (output instanceof AssetTransferOutput_1.AssetTransferOutput) {
                this.outputs.push(output);
            }
            else {
                const { assetType, amount, recipient } = output;
                this.outputs.push(new AssetTransferOutput_1.AssetTransferOutput({
                    recipient: codechain_primitives_1.AssetTransferAddress.ensure(recipient),
                    amount: U256_1.U256.ensure(amount),
                    assetType: H256_1.H256.ensure(assetType)
                }));
            }
        });
        return this;
    }
    /**
     * Get the output of the given index, of this transaction.
     * @param index An index indicating an output.
     * @returns An Asset.
     */
    getTransferredAsset(index) {
        if (index >= this.outputs.length) {
            throw Error("invalid output index");
        }
        const output = this.outputs[index];
        const { assetType, lockScriptHash, parameters, amount } = output;
        return new Asset_1.Asset({
            assetType,
            lockScriptHash,
            parameters,
            amount,
            transactionHash: this.hash(),
            transactionOutputIndex: index
        });
    }
    /**
     * Get the outputs of this transaction.
     * @returns An array of an Asset.
     */
    getTransferredAssets() {
        return _.range(this.outputs.length).map(i => this.getTransferredAsset(i));
    }
    /**
     * Get a hash of the transaction that doesn't contain the scripts. The hash
     * is used as a message to create a signature for a transaction.
     * @returns A hash.
     */
    hashWithoutScript(params) {
        const { networkId } = this;
        const { tag = { input: "all", output: "all" }, type = null, index = null } = params || {};
        let burns;
        let inputs;
        let outputs;
        if (tag.input === "all") {
            inputs = this.inputs.map(input => input.withoutScript());
            burns = this.burns.map(input => input.withoutScript());
        }
        else if (tag.input === "single") {
            if (typeof index !== "number") {
                throw Error(`Unexpected value of the index param: ${index}`);
            }
            if (type === "input") {
                inputs = [this.inputs[index].withoutScript()];
                burns = [];
            }
            else if (type === "burn") {
                inputs = [];
                burns = [this.burns[index].withoutScript()];
            }
            else {
                throw Error(`Unexpected value of the type param: ${type}`);
            }
        }
        else {
            throw Error(`Unexpected value of the tag input: ${tag.input}`);
        }
        if (tag.output === "all") {
            outputs = this.outputs;
        }
        else if (Array.isArray(tag.output)) {
            // NOTE: Remove duplicates by using Set
            outputs = Array.from(new Set(tag.output))
                .sort((a, b) => a - b)
                .map(i => this.outputs[i]);
        }
        else {
            throw Error(`Unexpected value of the tag output: ${tag.output}`);
        }
        return new H256_1.H256(utils_1.blake256WithKey(new AssetTransferTransaction({
            burns,
            inputs,
            outputs,
            networkId
        }).rlpBytes(), Buffer.from(utils_1.blake128(utils_1.encodeSignatureTag(tag)), "hex")));
    }
    /**
     * Get the asset address of an output.
     * @param index An index indicating the output.
     * @returns An asset address which is H256.
     */
    getAssetAddress(index) {
        const iv = new Uint8Array([
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            (index >> 56) & 0xff,
            (index >> 48) & 0xff,
            (index >> 40) & 0xff,
            (index >> 32) & 0xff,
            (index >> 24) & 0xff,
            (index >> 16) & 0xff,
            (index >> 8) & 0xff,
            index & 0xff
        ]);
        const shardId = this.outputs[index].shardId();
        const blake = utils_1.blake256WithKey(this.hash().value, iv);
        const shardPrefix = convertU16toHex(shardId);
        const prefix = `4100${shardPrefix}`;
        return new H256_1.H256(blake.replace(new RegExp(`^.{${prefix.length}}`), prefix));
    }
    /**
     * Convert to an AssetTransferTransaction JSON object.
     * @returns An AssetTransferTransaction JSON object.
     */
    toJSON() {
        const { networkId, burns, inputs, outputs } = this;
        return {
            type: this.type,
            data: {
                networkId,
                burns: burns.map(input => input.toJSON()),
                inputs: inputs.map(input => input.toJSON()),
                outputs: outputs.map(output => output.toJSON())
            }
        };
    }
}
exports.AssetTransferTransaction = AssetTransferTransaction;
function convertU16toHex(id) {
    const hi = ("0" + ((id >> 8) & 0xff).toString(16)).slice(-2);
    const lo = ("0" + (id & 0xff).toString(16)).slice(-2);
    return hi + lo;
}
