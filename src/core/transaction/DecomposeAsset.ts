import * as _ from "lodash";

import { blake128, blake256, blake256WithKey } from "../../utils";
import {
    Asset,
    AssetTransferAddress,
    AssetTransferInput,
    AssetTransferOutput,
    H256,
    U64
} from "../classes";
import { AssetTransaction, Transaction } from "../Transaction";
import { AssetTransferOutputValue, NetworkId } from "../types";

const RLP = require("rlp");

export class DecomposeAsset extends Transaction implements AssetTransaction {
    private readonly _transaction: AssetDecomposeTransaction;
    private readonly approvals: string[];
    public constructor(input: {
        input: AssetTransferInput;
        outputs: AssetTransferOutput[];
        networkId: NetworkId;
        approvals: string[];
    }) {
        super(input.networkId);

        this._transaction = new AssetDecomposeTransaction(input);
        this.approvals = input.approvals;
    }

    /**
     * Get the hash of an AssetDecomposeTransaction.
     * @returns A transaction hash.
     */
    public id(): H256 {
        return new H256(blake256(this._transaction.rlpBytes()));
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
                    input: this._transaction.input.withoutScript(),
                    outputs: this._transaction.outputs,
                    networkId: this._transaction.networkId
                }).rlpBytes(),
                Buffer.from(blake128(Buffer.from([0b00000011])), "hex")
            )
        );
    }

    public input(_index: number): AssetTransferInput | null {
        return this._transaction.input;
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
                this._transaction.outputs.push(output);
            } else {
                const { assetType, amount, recipient } = output;
                this._transaction.outputs.push(
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
        if (index >= this._transaction.outputs.length) {
            throw Error(`Invalid output index`);
        }
        const output = this._transaction.outputs[index];
        const { assetType, lockScriptHash, parameters, amount } = output;
        return new Asset({
            assetType,
            lockScriptHash,
            parameters,
            amount,
            transactionId: this.id(),
            transactionOutputIndex: index
        });
    }

    /**
     * Get the outputs of this transaction.
     * @returns An array of an Asset.
     */
    public getTransferredAssets(): Asset[] {
        return _.range(this._transaction.outputs.length).map(i =>
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
        const shardId = this._transaction.outputs[index].shardId();

        const blake = blake256WithKey(this.id().value, iv);
        const shardPrefix = convertU16toHex(shardId);
        const prefix = `4100${shardPrefix}`;
        return new H256(
            blake.replace(new RegExp(`^.{${prefix.length}}`), prefix)
        );
    }

    public type(): string {
        return "decomposeAsset";
    }

    protected actionToEncodeObject(): any[] {
        const transaction = this._transaction.toEncodeObject();
        const approvals = this.approvals;
        return [1, transaction, approvals];
    }

    protected actionToJSON(): any {
        const json = this._transaction.toJSON();
        json.approvals = this.approvals;
        return json;
    }
}

function convertU16toHex(id: number) {
    const hi: string = ("0" + ((id >> 8) & 0xff).toString(16)).slice(-2);
    const lo: string = ("0" + (id & 0xff).toString(16)).slice(-2);
    return hi + lo;
}

/**
 * Decompose assets. The sum of inputs must be whole supply of the asset.
 */
class AssetDecomposeTransaction {
    public readonly input: AssetTransferInput;
    public readonly outputs: AssetTransferOutput[];
    public readonly networkId: NetworkId;

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
    public toJSON(): any {
        const { input, outputs, networkId } = this;
        return {
            input: input.toJSON(),
            outputs: outputs.map(o => o.toJSON()),
            networkId
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
}
