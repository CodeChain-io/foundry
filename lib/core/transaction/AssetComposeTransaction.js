"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const lib_1 = require("codechain-primitives/lib");
const _ = require("lodash");
const utils_1 = require("../../utils");
const Asset_1 = require("../Asset");
const AssetScheme_1 = require("../AssetScheme");
const U256_1 = require("../U256");
const AssetMintOutput_1 = require("./AssetMintOutput");
const AssetTransferInput_1 = require("./AssetTransferInput");
const RLP = require("rlp");
/**
 * Compose assets.
 */
class AssetComposeTransaction {
    /**
     * @param params.networkId A network ID of the transaction.
     * @param params.shardId A shard ID of the transaction.
     * @param params.metadata A metadata of the asset.
     * @param params.registrar A registrar of the asset.
     * @param params.inputs A list of inputs of the transaction.
     * @param params.output An output of the transaction.
     */
    constructor(params) {
        this.type = "assetCompose";
        const { networkId, shardId, metadata, registrar, inputs, output } = params;
        this.networkId = networkId;
        this.shardId = shardId;
        this.metadata = metadata;
        this.registrar =
            registrar === null ? null : lib_1.PlatformAddress.ensure(registrar);
        this.inputs = inputs;
        this.output = new AssetMintOutput_1.AssetMintOutput(output);
    }
    /**
     * Create an AssetComposeTransaction from an AssetComposeTransaction JSON object.
     * @param obj An AssetComposeTransaction JSON object.
     * @returns An AssetComposeTransaction.
     */
    static fromJSON(obj) {
        const { data: { networkId, shardId, metadata, inputs, output, registrar } } = obj;
        return new this({
            networkId,
            shardId,
            metadata,
            registrar: registrar === null ? null : lib_1.PlatformAddress.ensure(registrar),
            inputs: inputs.map((input) => AssetTransferInput_1.AssetTransferInput.fromJSON(input)),
            output: AssetMintOutput_1.AssetMintOutput.fromJSON(output)
        });
    }
    /**
     * Convert to an AssetComposeTransaction JSON object.
     * @returns An AssetComposeTransaction JSON object.
     */
    toJSON() {
        return {
            type: this.type,
            data: {
                networkId: this.networkId,
                shardId: this.shardId,
                metadata: this.metadata,
                registrar: this.registrar,
                output: this.output.toJSON(),
                inputs: this.inputs.map(input => input.toJSON())
            }
        };
    }
    /**
     * Convert to an object for RLP encoding.
     */
    toEncodeObject() {
        return [
            6,
            this.networkId,
            this.shardId,
            this.metadata,
            this.registrar ? [this.registrar.toString()] : [],
            this.inputs.map(input => input.toEncodeObject()),
            this.output.lockScriptHash.toEncodeObject(),
            this.output.parameters.map(parameter => Buffer.from(parameter)),
            this.output.amount != null
                ? [this.output.amount.toEncodeObject()]
                : []
        ];
    }
    /**
     * Convert to RLP bytes.
     */
    rlpBytes() {
        return RLP.encode(this.toEncodeObject());
    }
    /**
     * Get the hash of an AssetComposeTransaction.
     * @returns A transaction hash.
     */
    hash() {
        return new lib_1.H256(utils_1.blake256(this.rlpBytes()));
    }
    /**
     * Get a hash of the transaction that doesn't contain the scripts. The hash
     * is used as a message to create a signature for a transaction.
     * @returns A hash.
     */
    hashWithoutScript(params) {
        const { tag = { input: "all", output: "all" }, index = null } = params || {};
        let inputs;
        if (tag.input === "all") {
            inputs = this.inputs.map(input => input.withoutScript());
        }
        else if (tag.input === "single") {
            if (typeof index !== "number") {
                throw Error(`Unexpected value of the index: ${index}`);
            }
            inputs = [this.inputs[index].withoutScript()];
        }
        else {
            throw Error(`Unexpected value of the tag input: ${tag.input}`);
        }
        let output;
        if (tag.output === "all") {
            output = this.output;
        }
        else if (Array.isArray(tag.output) && tag.output.length === 0) {
            // NOTE: An empty array is allowed only
            output = new AssetMintOutput_1.AssetMintOutput({
                lockScriptHash: new lib_1.H160("0000000000000000000000000000000000000000"),
                parameters: [],
                amount: null
            });
        }
        else {
            throw Error(`Unexpected value of the tag output: ${tag.output}`);
        }
        const { networkId, shardId, metadata, registrar } = this;
        return new lib_1.H256(utils_1.blake256WithKey(new AssetComposeTransaction({
            networkId,
            shardId,
            metadata,
            registrar,
            inputs,
            output
        }).rlpBytes(), Buffer.from(lib_1.blake128(utils_1.encodeSignatureTag(tag)), "hex")));
    }
    /**
     * Add an AssetTransferInput to spend.
     * @param inputs An array of either an AssetTransferInput or an Asset.
     * @returns The modified AssetComposeTransaction.
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
                throw Error(`Expected an array of either AssetTransferInput or Asset but found ${input} at ${index}`);
            }
        });
        return this;
    }
    /**
     * Get the output of this transaction.
     * @returns An Asset.
     */
    getComposedAsset() {
        const { lockScriptHash, parameters, amount } = this.output;
        if (amount === null) {
            throw Error("not implemented");
        }
        return new Asset_1.Asset({
            assetType: this.getAssetSchemeAddress(),
            lockScriptHash,
            parameters,
            amount: amount == null ? U256_1.U256.ensure(U256_1.U256.MAX_VALUE) : amount,
            transactionHash: this.hash(),
            transactionOutputIndex: 0
        });
    }
    /**
     * Get the asset scheme of this transaction.
     * @return An AssetScheme.
     */
    getAssetScheme() {
        const { networkId, shardId, metadata, inputs, output: { amount }, registrar } = this;
        // FIXME: need U64 to be implemented or use U256
        if (amount == null) {
            throw Error("not implemented");
        }
        return new AssetScheme_1.AssetScheme({
            networkId,
            shardId,
            metadata,
            amount,
            registrar,
            pool: _.toPairs(
            // NOTE: Get the sum of each asset type
            inputs.reduce((acc, input) => {
                const { assetType, amount: assetAmount } = input.prevOut;
                // FIXME: Check integer overflow
                acc[assetType.value] = U256_1.U256.plus(acc[assetType.value], assetAmount);
                return acc;
            }, {})).map(([assetType, assetAmount]) => ({
                assetType: lib_1.H256.ensure(assetType),
                amount: U256_1.U256.ensure(assetAmount)
            }))
        });
    }
    /**
     * Get the address of the asset scheme. An asset scheme address equals to an
     * asset type value.
     * @returns An asset scheme address which is H256.
     */
    getAssetSchemeAddress() {
        const { shardId } = this;
        const blake = utils_1.blake256WithKey(this.hash().value, new Uint8Array([
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0xff,
            0xff,
            0xff,
            0xff,
            0xff,
            0xff,
            0xff,
            0xff
        ]));
        const shardPrefix = convertU16toHex(shardId);
        const prefix = `5300${shardPrefix}`;
        return new lib_1.H256(blake.replace(new RegExp(`^.{${prefix.length}}`), prefix));
    }
    /**
     * Get the asset address of the output.
     * @returns An asset address which is H256.
     */
    getAssetAddress() {
        const { shardId } = this;
        const blake = utils_1.blake256WithKey(this.hash().value, new Uint8Array([
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00
        ]));
        const shardPrefix = convertU16toHex(shardId);
        const prefix = `4100${shardPrefix}`;
        return new lib_1.H256(blake.replace(new RegExp(`^.{${prefix.length}}`), prefix));
    }
}
exports.AssetComposeTransaction = AssetComposeTransaction;
function convertU16toHex(id) {
    const hi = ("0" + ((id >> 8) & 0xff).toString(16)).slice(-2);
    const lo = ("0" + (id & 0xff).toString(16)).slice(-2);
    return hi + lo;
}
