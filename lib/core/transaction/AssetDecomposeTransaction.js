"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const lib_1 = require("codechain-primitives/lib");
const _ = require("lodash");
const Asset_1 = require("../Asset");
const U256_1 = require("../U256");
const AssetTransferInput_1 = require("./AssetTransferInput");
const AssetTransferOutput_1 = require("./AssetTransferOutput");
const RLP = require("rlp");
/**
 * Decompose assets. The sum of inputs must be whole supply of the asset.
 */
class AssetDecomposeTransaction {
    /**
     * @param params.inputs An array of AssetTransferInput to decompose.
     * @param params.outputs An array of AssetTransferOutput to create.
     * @param params.networkId A network ID of the transaction.
     */
    constructor(params) {
        this.type = "assetDecompose";
        this.input = params.input;
        this.outputs = params.outputs;
        this.networkId = params.networkId;
    }
    /**
     * Create an AssetDecomposeTransaction from an AssetDecomposeTransaction JSON object.
     * @param obj An AssetDecomposeTransaction JSON object.
     * @returns An AssetDecomposeTransaction.
     */
    static fromJSON(obj) {
        const { data: { input, outputs, networkId } } = obj;
        return new this({
            input: AssetTransferInput_1.AssetTransferInput.fromJSON(input),
            outputs: outputs.map((o) => AssetTransferInput_1.AssetTransferInput.fromJSON(o)),
            networkId
        });
    }
    /**
     * Convert to an AssetDecomposeTransaction JSON object.
     * @returns An AssetDecomposeTransaction JSON object.
     */
    toJSON() {
        const { type, input, outputs, networkId } = this;
        return {
            type,
            data: {
                input: input.toJSON(),
                outputs: outputs.map(o => o.toJSON()),
                networkId
            }
        };
    }
    /**
     * Convert to an object for RLP encoding.
     */
    toEncodeObject() {
        return [
            7,
            this.networkId,
            this.input.toEncodeObject(),
            this.outputs.map(o => o.toEncodeObject())
        ];
    }
    /**
     * Convert to RLP bytes.
     */
    rlpBytes() {
        return RLP.encode(this.toEncodeObject());
    }
    /**
     * Get the hash of an AssetDecomposeTransaction.
     * @returns A transaction hash.
     */
    hash() {
        return new lib_1.H256(lib_1.blake256(this.rlpBytes()));
    }
    /**
     * Get a hash of the transaction that doesn't contain the scripts. The hash
     * is used as a message to create a signature for a transaction.
     * @returns A hash.
     */
    hashWithoutScript() {
        // Since there is only one input, the signature tag byte must be 0b00000011.
        return new lib_1.H256(lib_1.blake256WithKey(new AssetDecomposeTransaction({
            input: this.input.withoutScript(),
            outputs: this.outputs,
            networkId: this.networkId
        }).rlpBytes(), Buffer.from(lib_1.blake128(Buffer.from([0b00000011])), "hex")));
    }
    /**
     * Add AssetTransferOutputs to create.
     * @param outputs An array of either an AssetTransferOutput or an object
     * containing amount, asset type, and recipient.
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
                    recipient: lib_1.AssetTransferAddress.ensure(recipient),
                    amount: U256_1.U256.ensure(amount),
                    assetType: lib_1.H256.ensure(assetType)
                }));
            }
        });
    }
    /**
     * Get the output of the given index, of this transaction.
     * @param index An index indicating an output.
     * @returns An Asset.
     */
    getTransferredAsset(index) {
        if (index >= this.outputs.length) {
            throw Error(`Invalid output index`);
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
        const blake = lib_1.blake256WithKey(this.hash().value, iv);
        const shardPrefix = convertU16toHex(shardId);
        const prefix = `4100${shardPrefix}`;
        return new lib_1.H256(blake.replace(new RegExp(`^.{${prefix.length}}`), prefix));
    }
}
exports.AssetDecomposeTransaction = AssetDecomposeTransaction;
function convertU16toHex(id) {
    const hi = ("0" + ((id >> 8) & 0xff).toString(16)).slice(-2);
    const lo = ("0" + (id & 0xff).toString(16)).slice(-2);
    return hi + lo;
}
