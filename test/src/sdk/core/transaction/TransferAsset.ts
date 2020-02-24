import * as _ from "lodash";

import { U64Value } from "codechain-primitives/lib";
import {
    blake128,
    blake256,
    blake256WithKey,
    encodeSignatureTag,
    SignatureTag
} from "../../utils";
import { Asset } from "../Asset";
import {
    AssetAddress,
    H160,
    H256,
    Order,
    OrderOnTransfer,
    U64
} from "../classes";
import { AssetTransaction, Transaction } from "../Transaction";
import { AssetTransferOutputValue, NetworkId } from "../types";
import {
    AssetTransferInput,
    AssetTransferInputJSON
} from "./AssetTransferInput";
import {
    AssetTransferOutput,
    AssetTransferOutputJSON
} from "./AssetTransferOutput";
import { OrderOnTransferJSON } from "./OrderOnTransfer";

const RLP = require("rlp");

export interface AssetTransferTransactionJSON {
    networkId: string;
    burns: AssetTransferInputJSON[];
    inputs: AssetTransferInputJSON[];
    outputs: AssetTransferOutputJSON[];
    orders: OrderOnTransferJSON[];
}
export interface TransferAssetActionJSON extends AssetTransferTransactionJSON {
    metadata: string;
    approvals: string[];
    expiration: number | null;
}

export class TransferAsset extends Transaction implements AssetTransaction {
    private readonly _transaction: AssetTransferTransaction;
    private readonly approvals: string[];
    private readonly metadata: string;
    private readonly expiration: number | null;

    public constructor(input: {
        burns: AssetTransferInput[];
        inputs: AssetTransferInput[];
        outputs: AssetTransferOutput[];
        orders: OrderOnTransfer[];
        networkId: NetworkId;
        metadata: string | object;
        approvals: string[];
        expiration: number | null;
    }) {
        super(input.networkId);

        this._transaction = new AssetTransferTransaction(input);
        this.metadata =
            typeof input.metadata === "string"
                ? input.metadata
                : JSON.stringify(input.metadata);
        this.approvals = input.approvals;
        this.expiration = input.expiration;
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
     * Add an AssetTransferInput to burn.
     * @param burns An array of either an AssetTransferInput or an Asset.
     * @returns The TransferAsset, which is modified by adding them.
     */
    public addBurns(
        burns: AssetTransferInput | Asset | Array<AssetTransferInput | Asset>,
        ...rest: Array<AssetTransferInput | Asset>
    ): TransferAsset {
        if (!Array.isArray(burns)) {
            burns = [burns, ...rest];
        }
        burns.forEach((burn: AssetTransferInput | Asset, index: number) => {
            if (burn instanceof AssetTransferInput) {
                this._transaction.burns.push(burn);
            } else if (burn instanceof Asset) {
                this._transaction.burns.push(burn.createTransferInput());
            } else {
                throw Error(
                    `Expected burn param to be either AssetTransferInput or Asset but found ${burn} at ${index}`
                );
            }
        });
        return this;
    }

    public burns(): AssetTransferInput[] {
        return this._transaction.burns;
    }

    public burn(index: number): AssetTransferInput | null {
        if (this._transaction.burns.length <= index) {
            return null;
        }
        return this._transaction.burns[index];
    }

    /**
     * Add an AssetTransferInput to spend.
     * @param inputs An array of either an AssetTransferInput or an Asset.
     * @returns The TransferAsset, which is modified by adding them.
     */
    public addInputs(
        inputs: AssetTransferInput | Asset | Array<AssetTransferInput | Asset>,
        ...rest: Array<AssetTransferInput | Asset>
    ): TransferAsset {
        if (!Array.isArray(inputs)) {
            inputs = [inputs, ...rest];
        }
        inputs.forEach((input: AssetTransferInput | Asset, index: number) => {
            if (input instanceof AssetTransferInput) {
                this._transaction.inputs.push(input);
            } else if (input instanceof Asset) {
                this._transaction.inputs.push(input.createTransferInput());
            } else {
                throw Error(
                    `Expected input param to be either AssetTransferInput or Asset but found ${input} at ${index}`
                );
            }
        });
        return this;
    }

    public inputs(): AssetTransferInput[] {
        return this._transaction.inputs;
    }

    public input(index: number): AssetTransferInput | null {
        if (this._transaction.inputs.length <= index) {
            return null;
        }
        return this._transaction.inputs[index];
    }

    /**
     * Add an AssetTransferOutput to create.
     * @param outputs An array of either an AssetTransferOutput or an object
     * that has quantity, assetType and recipient values.
     * @param output.quantity Asset quantity of the output.
     * @param output.assetType An asset type of the output.
     * @param output.recipient A recipient of the output.
     */
    public addOutputs(
        outputs: AssetTransferOutputValue | Array<AssetTransferOutputValue>,
        ...rest: Array<AssetTransferOutputValue>
    ): TransferAsset {
        if (!Array.isArray(outputs)) {
            outputs = [outputs, ...rest];
        }
        outputs.forEach((output: AssetTransferOutputValue) => {
            if (output instanceof AssetTransferOutput) {
                this._transaction.outputs.push(output);
            } else {
                const { assetType, shardId, quantity, recipient } = output;
                this._transaction.outputs.push(
                    new AssetTransferOutput({
                        recipient: AssetAddress.ensure(recipient),
                        quantity: U64.ensure(quantity),
                        assetType: H160.ensure(assetType),
                        shardId
                    })
                );
            }
        });
        return this;
    }

    public outputs(): AssetTransferOutput[] {
        return this._transaction.outputs;
    }

    public output(index: number): AssetTransferOutput | null {
        if (this._transaction.outputs.length <= index) {
            return null;
        }
        return this._transaction.outputs[index];
    }

    /**
     * Add an Order to create.
     * @param params.order An order to apply to the transfer transaction.
     * @param params.spentQuantity A spent quantity of the asset to give(from) while transferring.
     * @param params.inputIndices The indices of inputs affected by the order
     * @param params.outputIndices The indices of outputs affected by the order
     */
    public addOrder(params: {
        order: Order;
        spentQuantity: U64Value;
        inputFromIndices: number[];
        inputFeeIndices: number[];
        outputFromIndices: number[];
        outputToIndices: number[];
        outputOwnedFeeIndices: number[];
        outputTransferredFeeIndices: number[];
    }) {
        const {
            order,
            spentQuantity,
            inputFromIndices = [],
            inputFeeIndices = [],
            outputFromIndices = [],
            outputToIndices = [],
            outputOwnedFeeIndices = [],
            outputTransferredFeeIndices = []
        } = params;
        if (inputFromIndices.length === 0) {
            throw Error(`inputFromIndices should not be empty`);
        }
        const inputIndices = inputFromIndices.concat(inputFeeIndices);
        const outputIndices = outputFromIndices
            .concat(outputToIndices)
            .concat(outputOwnedFeeIndices)
            .concat(outputTransferredFeeIndices);

        for (const orderOnTx of this._transaction.orders) {
            const setInputs = new Set(
                orderOnTx.inputFromIndices.concat(orderOnTx.inputFeeIndices)
            );
            const setOutputs = new Set(
                orderOnTx.outputFromIndices
                    .concat(orderOnTx.outputToIndices)
                    .concat(orderOnTx.outputOwnedFeeIndices)
                    .concat(orderOnTx.outputTransferredFeeIndices)
            );
            const inputIntersection = inputIndices.filter(x =>
                setInputs.has(x)
            );
            const outputIntersection = outputIndices.filter(x =>
                setOutputs.has(x)
            );
            if (inputIntersection.length > 0 || outputIntersection.length > 0) {
                throw Error(
                    `inputIndices and outputIndices should not intersect with other orders: ${orderOnTx}`
                );
            }
        }

        // NOTE: Do not empty any of the indices
        this._transaction.orders.push(
            new OrderOnTransfer({
                order,
                spentQuantity: U64.ensure(spentQuantity),
                inputFromIndices,
                inputFeeIndices,
                outputFromIndices,
                outputToIndices,
                outputOwnedFeeIndices,
                outputTransferredFeeIndices
            })
        );
        return this;
    }

    public orders(): OrderOnTransfer[] {
        return this._transaction.orders;
    }

    /**
     * Get the output of the given index, of this transaction.
     * @param index An index indicating an output.
     * @returns An Asset.
     */
    public getTransferredAsset(index: number): Asset {
        if (index >= this._transaction.outputs.length) {
            throw Error("invalid output index");
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
        const { networkId } = this._transaction;
        const {
            tag = { input: "all", output: "all" } as SignatureTag,
            type = null,
            index = null
        } = params || {};
        let burns: AssetTransferInput[];
        let inputs: AssetTransferInput[];
        let outputs: AssetTransferOutput[];

        if (
            this._transaction.orders.length > 0 &&
            (tag.input !== "all" || tag.output !== "all")
        ) {
            throw Error(`Partial signing is unavailable with orders`);
        }
        if (tag.input === "all") {
            inputs = this._transaction.inputs.map(input =>
                input.withoutScript()
            );
            burns = this._transaction.burns.map(input => input.withoutScript());
        } else if (tag.input === "single") {
            if (typeof index !== "number") {
                throw Error(`Unexpected value of the index param: ${index}`);
            }
            if (type === "input") {
                inputs = [this._transaction.inputs[index].withoutScript()];
                burns = [];
            } else if (type === "burn") {
                inputs = [];
                burns = [this._transaction.burns[index].withoutScript()];
            } else {
                throw Error(`Unexpected value of the type param: ${type}`);
            }
        } else {
            throw Error(`Unexpected value of the tag input: ${tag.input}`);
        }
        if (tag.output === "all") {
            outputs = this._transaction.outputs;
        } else if (Array.isArray(tag.output)) {
            // NOTE: Remove duplicates by using Set
            outputs = Array.from(new Set(tag.output))
                .sort((a, b) => a - b)
                .map(i => this._transaction.outputs[i]);
        } else {
            throw Error(`Unexpected value of the tag output: ${tag.output}`);
        }
        return new H256(
            blake256WithKey(
                new AssetTransferTransaction({
                    burns,
                    inputs,
                    outputs,
                    orders: this._transaction.orders,
                    networkId
                }).rlpBytes(),
                Buffer.from(blake128(encodeSignatureTag(tag)), "hex")
            )
        );
    }

    public type(): string {
        return "transferAsset";
    }

    protected actionToEncodeObject(): any[] {
        const { expiration, metadata, approvals } = this;
        const encoded: any[] = this._transaction.toEncodeObject();
        encoded.push(metadata);
        encoded.push(approvals);
        if (expiration == null) {
            encoded.push([]);
        } else {
            encoded.push([expiration]);
        }
        return encoded;
    }

    protected actionToJSON(): TransferAssetActionJSON {
        const json = this._transaction.toJSON();
        return {
            ...json,
            metadata: this.metadata,
            approvals: this.approvals,
            expiration: this.expiration
        };
    }
}

interface AssetTransferTransactionData {
    burns: AssetTransferInput[];
    inputs: AssetTransferInput[];
    outputs: AssetTransferOutput[];
    orders: OrderOnTransfer[];
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
 * types' quantity have been summed, the sum of inputs and the sum of outputs
 * must be identical.
 */
class AssetTransferTransaction {
    public readonly burns: AssetTransferInput[];
    public readonly inputs: AssetTransferInput[];
    public readonly outputs: AssetTransferOutput[];
    public readonly orders: OrderOnTransfer[];
    public readonly networkId: NetworkId;

    /**
     * @param params.burns An array of AssetTransferInput to burn.
     * @param params.inputs An array of AssetTransferInput to spend.
     * @param params.outputs An array of AssetTransferOutput to create.
     * @param params.networkId A network ID of the transaction.
     */
    constructor(params: AssetTransferTransactionData) {
        const { burns, inputs, outputs, orders, networkId } = params;
        this.burns = burns;
        this.inputs = inputs;
        this.outputs = outputs;
        this.orders = orders;
        this.networkId = networkId;
    }

    /**
     * Convert to an AssetTransferTransaction JSON object.
     * @returns An AssetTransferTransaction JSON object.
     */
    public toJSON(): AssetTransferTransactionJSON {
        const { networkId, burns, inputs, outputs, orders } = this;
        return {
            networkId,
            burns: burns.map(input => input.toJSON()),
            inputs: inputs.map(input => input.toJSON()),
            outputs: outputs.map(output => output.toJSON()),
            orders: orders.map(order => order.toJSON())
        };
    }

    /**
     * Convert to an object for RLP encoding.
     */
    public toEncodeObject() {
        return [
            0x14,
            this.networkId,
            this.burns.map(input => input.toEncodeObject()),
            this.inputs.map(input => input.toEncodeObject()),
            this.outputs.map(output => output.toEncodeObject()),
            this.orders.map(order => order.toEncodeObject())
        ];
    }

    /**
     * Convert to RLP bytes.
     */
    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }
}
