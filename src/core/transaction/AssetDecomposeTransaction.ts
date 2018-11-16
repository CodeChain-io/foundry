import {
    AssetTransferAddress,
    blake128,
    blake256,
    blake256WithKey,
    H256
} from "codechain-primitives/lib";
import * as _ from "lodash";

import { Asset } from "../Asset";
import { AssetTransferOutputValue, NetworkId } from "../types";
import { U64 } from "../U64";
import {
    AssetTransferInput,
    AssetTransferInputJSON
} from "./AssetTransferInput";
import {
    AssetTransferOutput,
    AssetTransferOutputJSON
} from "./AssetTransferOutput";

const RLP = require("rlp");

export interface AssetDecomposeTransactionJSON {
    type: "assetDecompose";
    data: {
        input: AssetTransferInputJSON;
        outputs: AssetTransferOutputJSON[];
        networkId: NetworkId;
    };
}

/**
 * Decompose assets. The sum of inputs must be whole supply of the asset.
 */
export class AssetDecomposeTransaction {
    /**
     * Create an AssetDecomposeTransaction from an AssetDecomposeTransaction JSON object.
     * @param obj An AssetDecomposeTransaction JSON object.
     * @returns An AssetDecomposeTransaction.
     */
    public static fromJSON(obj: AssetDecomposeTransactionJSON) {
        const {
            data: { input, outputs, networkId }
        } = obj;
        return new this({
            input: AssetTransferInput.fromJSON(input),
            outputs: outputs.map(o => AssetTransferOutput.fromJSON(o)),
            networkId
        });
    }

    public readonly input: AssetTransferInput;
    public readonly outputs: AssetTransferOutput[];
    public readonly networkId: NetworkId;
    public readonly type = "assetDecompose";

    /**
     * @param params.inputs An array of AssetTransferInput to decompose.
     * @param params.outputs An array of AssetTransferOutput to create.
     * @param params.networkId A network ID of the transaction.
     */
    constructor(params: {
        input: AssetTransferInput;
        outputs: AssetTransferOutput[];
        networkId: NetworkId;
    }) {
        this.input = params.input;
        this.outputs = params.outputs;
        this.networkId = params.networkId;
    }

    /**
     * Convert to an AssetDecomposeTransaction JSON object.
     * @returns An AssetDecomposeTransaction JSON object.
     */
    public toJSON(): AssetDecomposeTransactionJSON {
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
    public toEncodeObject() {
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
    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    /**
     * Get the hash of an AssetDecomposeTransaction.
     * @returns A transaction hash.
     */
    public hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }

    /**
     * Get a hash of the transaction that doesn't contain the scripts. The hash
     * is used as a message to create a signature for a transaction.
     * @returns A hash.
     */
    public hashWithoutScript(): H256 {
        // Since there is only one input, the signature tag byte must be 0b00000011.
        return new H256(
            blake256WithKey(
                new AssetDecomposeTransaction({
                    input: this.input.withoutScript(),
                    outputs: this.outputs,
                    networkId: this.networkId
                }).rlpBytes(),
                Buffer.from(blake128(Buffer.from([0b00000011])), "hex")
            )
        );
    }

    /**
     * Add AssetTransferOutputs to create.
     * @param outputs An array of either an AssetTransferOutput or an object
     * containing amount, asset type, and recipient.
     */
    public addOutputs(
        outputs: AssetTransferOutputValue | Array<AssetTransferOutputValue>,
        ...rest: Array<AssetTransferOutputValue>
    ) {
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
                        amount: U64.ensure(amount),
                        assetType: H256.ensure(assetType)
                    })
                );
            }
        });
    }

    /**
     * Get the output of the given index, of this transaction.
     * @param index An index indicating an output.
     * @returns An Asset.
     */
    public getTransferredAsset(index: number): Asset {
        if (index >= this.outputs.length) {
            throw Error(`Invalid output index`);
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
}

function convertU16toHex(id: number) {
    const hi: string = ("0" + ((id >> 8) & 0xff).toString(16)).slice(-2);
    const lo: string = ("0" + (id & 0xff).toString(16)).slice(-2);
    return hi + lo;
}
