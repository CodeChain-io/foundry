import { AssetTransferAddress } from "codechain-primitives";
import * as _ from "lodash";

import {
    blake128,
    blake256,
    blake256WithKey,
    encodeSignatureTag,
    SignatureTag
} from "../../utils";
import { Asset } from "../Asset";
import { H256 } from "../H256";
import { AssetTransferOutputValue, NetworkId } from "../types";
import { U256 } from "../U256";
import { AssetTransferInput } from "./AssetTransferInput";
import { AssetTransferOutput } from "./AssetTransferOutput";

const RLP = require("rlp");

export interface AssetTransferTransactionData {
    burns: AssetTransferInput[];
    inputs: AssetTransferInput[];
    outputs: AssetTransferOutput[];
    networkId: NetworkId;
}
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
export class AssetTransferTransaction {
    /** Create an AssetTransferTransaction from an AssetTransferTransaction JSON object.
     * @param obj An AssetTransferTransaction JSON object.
     * @returns An AssetTransferTransaction.
     */
    public static fromJSON(obj: any) {
        const {
            data: { networkId, burns, inputs, outputs }
        } = obj;
        return new this({
            burns: burns.map((input: any) =>
                AssetTransferInput.fromJSON(input)
            ),
            inputs: inputs.map((input: any) =>
                AssetTransferInput.fromJSON(input)
            ),
            outputs: outputs.map((output: any) =>
                AssetTransferOutput.fromJSON(output)
            ),
            networkId
        });
    }
    public readonly burns: AssetTransferInput[];
    public readonly inputs: AssetTransferInput[];
    public readonly outputs: AssetTransferOutput[];
    public readonly networkId: NetworkId;
    public readonly type = "assetTransfer";

    /**
     * @param params.burns An array of AssetTransferInput to burn.
     * @param params.inputs An array of AssetTransferInput to spend.
     * @param params.outputs An array of AssetTransferOutput to create.
     * @param params.networkId A network ID of the transaction.
     */
    constructor(params: AssetTransferTransactionData) {
        const { burns, inputs, outputs, networkId } = params;
        this.burns = burns;
        this.inputs = inputs;
        this.outputs = outputs;
        this.networkId = networkId;
    }

    /**
     * Convert to an object for RLP encoding.
     */
    public toEncodeObject() {
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
    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    /**
     * Get the hash of an AssetTransferTransaction.
     * @returns A transaction hash.
     */
    public hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }

    /**
     * Add an AssetTransferInput to burn.
     * @param burns An array of either an AssetTransferInput or an Asset.
     * @returns The AssetTransferTransaction, which is modified by adding them.
     */
    public addBurns(
        burns: AssetTransferInput | Asset | Array<AssetTransferInput | Asset>,
        ...rest: Array<AssetTransferInput | Asset>
    ): AssetTransferTransaction {
        if (!Array.isArray(burns)) {
            burns = [burns, ...rest];
        }
        burns.forEach((burn, index) => {
            if (burn instanceof AssetTransferInput) {
                this.burns.push(burn);
            } else if (burn instanceof Asset) {
                this.burns.push(burn.createTransferInput());
            } else {
                throw Error(
                    `Expected burn param to be either AssetTransferInput or Asset but found ${burn} at ${index}`
                );
            }
        });
        return this;
    }

    /**
     * Add an AssetTransferInput to spend.
     * @param inputs An array of either an AssetTransferInput or an Asset.
     * @returns The AssetTransferTransaction, which is modified by adding them.
     */
    public addInputs(
        inputs: AssetTransferInput | Asset | Array<AssetTransferInput | Asset>,
        ...rest: Array<AssetTransferInput | Asset>
    ): AssetTransferTransaction {
        if (!Array.isArray(inputs)) {
            inputs = [inputs, ...rest];
        }
        inputs.forEach((input, index) => {
            if (input instanceof AssetTransferInput) {
                this.inputs.push(input);
            } else if (input instanceof Asset) {
                this.inputs.push(input.createTransferInput());
            } else {
                throw Error(
                    `Expected input param to be either AssetTransferInput or Asset but found ${input} at ${index}`
                );
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
    public addOutputs(
        outputs: AssetTransferOutputValue | Array<AssetTransferOutputValue>,
        ...rest: Array<AssetTransferOutputValue>
    ): AssetTransferTransaction {
        if (!Array.isArray(outputs)) {
            outputs = [outputs, ...rest];
        }
        outputs.forEach(output => {
            if (output instanceof AssetTransferOutput) {
                this.outputs.push(output);
            } else {
                const { assetType, amount, recipient } = output;
                this.outputs.push(
                    new AssetTransferOutput({
                        recipient: AssetTransferAddress.ensure(recipient),
                        amount: U256.ensure(amount),
                        assetType: H256.ensure(assetType)
                    })
                );
            }
        });
        return this;
    }

    /**
     * Get the output of the given index, of this transaction.
     * @param index An index indicating an output.
     * @returns An Asset.
     */
    public getTransferredAsset(index: number): Asset {
        if (index >= this.outputs.length) {
            throw Error("invalid output index");
        }
        const output = this.outputs[index];
        const { assetType, lockScriptHash, parameters, amount } = output;
        return new Asset({
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
    public getTransferredAssets(): Asset[] {
        return _.range(this.outputs.length).map(i =>
            this.getTransferredAsset(i)
        );
    }

    /**
     * Get a hash of the transaction that doesn't contain the scripts. The hash
     * is used as a message to create a signature for a transaction.
     * @returns A hash.
     */
    public hashWithoutScript(params?: {
        tag: SignatureTag;
        type: "input" | "burn";
        index: number;
    }): H256 {
        const { networkId } = this;
        const {
            tag = { input: "all", output: "all" } as SignatureTag,
            type = null,
            index = null
        } = params || {};
        let burns: AssetTransferInput[];
        let inputs: AssetTransferInput[];
        let outputs: AssetTransferOutput[];
        if (tag.input === "all") {
            inputs = this.inputs.map(input => input.withoutScript());
            burns = this.burns.map(input => input.withoutScript());
        } else if (tag.input === "single") {
            if (typeof index !== "number") {
                throw Error(`Unexpected value of the index param: ${index}`);
            }
            if (type === "input") {
                inputs = [this.inputs[index].withoutScript()];
                burns = [];
            } else if (type === "burn") {
                inputs = [];
                burns = [this.burns[index].withoutScript()];
            } else {
                throw Error(`Unexpected value of the type param: ${type}`);
            }
        } else {
            throw Error(`Unexpected value of the tag input: ${tag.input}`);
        }
        if (tag.output === "all") {
            outputs = this.outputs;
        } else if (Array.isArray(tag.output)) {
            // NOTE: Remove duplicates by using Set
            outputs = Array.from(new Set(tag.output))
                .sort((a, b) => a - b)
                .map(i => this.outputs[i]);
        } else {
            throw Error(`Unexpected value of the tag output: ${tag.output}`);
        }
        return new H256(
            blake256WithKey(
                new AssetTransferTransaction({
                    burns,
                    inputs,
                    outputs,
                    networkId
                }).rlpBytes(),
                Buffer.from(blake128(encodeSignatureTag(tag)), "hex")
            )
        );
    }

    /**
     * Get the asset address of an output.
     * @param index An index indicating the output.
     * @returns An asset address which is H256.
     */
    public getAssetAddress(index: number): H256 {
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

        const blake = blake256WithKey(this.hash().value, iv);
        const shardPrefix = convertU16toHex(shardId);
        const prefix = `4100${shardPrefix}`;
        return new H256(
            blake.replace(new RegExp(`^.{${prefix.length}}`), prefix)
        );
    }

    /**
     * Convert to an AssetTransferTransaction JSON object.
     * @returns An AssetTransferTransaction JSON object.
     */
    public toJSON() {
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

function convertU16toHex(id: number) {
    const hi: string = ("0" + ((id >> 8) & 0xff).toString(16)).slice(-2);
    const lo: string = ("0" + (id & 0xff).toString(16)).slice(-2);
    return hi + lo;
}
