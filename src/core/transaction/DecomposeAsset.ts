import * as _ from "lodash";

import { blake128, blake256, blake256WithKey } from "../../utils";
import {
    Asset,
    AssetTransferAddress,
    AssetTransferInput,
    AssetTransferOutput,
    H160,
    H256,
    U64
} from "../classes";
import { AssetTransaction, Transaction } from "../Transaction";
import { AssetTransferOutputValue, NetworkId } from "../types";
import { AssetTransferInputJSON } from "./AssetTransferInput";
import { AssetTransferOutputJSON } from "./AssetTransferOutput";

const RLP = require("rlp");

export interface AssetDecomposeTransactionJSON {
    input: AssetTransferInputJSON;
    outputs: AssetTransferOutputJSON[];
    networkId: string;
}

export interface DecomposeAssetActionJSON
    extends AssetDecomposeTransactionJSON {
    approvals: string[];
}

export class DecomposeAsset extends Transaction implements AssetTransaction {
    private readonly _transaction: AssetDecomposeTransaction;
    private readonly approvals: string[];
    public constructor(input: {
        input: AssetTransferInput;
        outputs: AssetTransferOutput[];
        networkId: NetworkId;
        approvals: string[];
    }) {
        throw Error("DecomposeAsset is disabled");
        super(input.networkId);

        this._transaction = new AssetDecomposeTransaction(input);
        this.approvals = input.approvals;
    }

    /**
     * Get the tracker of an AssetDecomposeTransaction.
     * @returns A transaction tracker.
     */
    public tracker(): H256 {
        return new H256(blake256(this._transaction.rlpBytes()));
    }

    /**
     * Add an approval to transaction.
     * @param approval An approval
     */
    public addApproval(approval: string) {
        this.approvals.push(approval);
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
     * containing quantity, asset type, and recipient.
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
                const { assetType, shardId, quantity, recipient } = output;
                this._transaction.outputs.push(
                    new AssetTransferOutput({
                        recipient: AssetTransferAddress.ensure(recipient),
                        quantity: U64.ensure(quantity),
                        assetType: H160.ensure(assetType),
                        shardId
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
        const {
            assetType,
            shardId,
            lockScriptHash,
            parameters,
            quantity
        } = output;
        return new Asset({
            assetType,
            shardId,
            lockScriptHash,
            parameters,
            quantity,
            tracker: this.tracker(),
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

    public type(): string {
        return "decomposeAsset";
    }

    protected actionToEncodeObject(): any[] {
        const encoded: any[] = this._transaction.toEncodeObject();
        encoded.push(this.approvals);
        return encoded;
    }

    protected actionToJSON(): DecomposeAssetActionJSON {
        const json = this._transaction.toJSON();
        return {
            ...json,
            approvals: this.approvals
        };
    }
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
    public toJSON(): AssetDecomposeTransactionJSON {
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
            0x17,
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
