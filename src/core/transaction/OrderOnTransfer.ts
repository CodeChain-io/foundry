import { U64 } from "codechain-primitives";

import { Order, OrderJSON } from "./Order";

const RLP = require("rlp");

export interface OrderOnTransferJSON {
    order: OrderJSON;
    spentQuantity: string;
    inputIndices: number[];
    outputIndices: number[];
}

export interface OrderOnTransferData {
    order: Order;
    spentQuantity: U64;
    inputIndices: number[];
    outputIndices: number[];
}

export class OrderOnTransfer {
    /**
     * Create an Order from an OrderJSON object.
     * @param data An OrderJSON object.
     * @returns An Order.
     */
    public static fromJSON(data: OrderOnTransferJSON) {
        const { order, spentQuantity, inputIndices, outputIndices } = data;
        return new OrderOnTransfer({
            order: Order.fromJSON(order),
            spentQuantity: U64.ensure(spentQuantity),
            inputIndices,
            outputIndices
        });
    }

    public readonly order: Order;
    public readonly spentQuantity: U64;
    public inputIndices: number[];
    public outputIndices: number[];

    /**
     * @param params.order An order to apply to the transfer transaction.
     * @param data.spentQuantity A spent quantity of the asset to give(from) while transferring.
     * @param data.inputIndices The indices of inputs affected by the order
     * @param data.outputIndices The indices of outputs affected by the order
     */
    constructor(data: OrderOnTransferData) {
        const { order, spentQuantity, inputIndices, outputIndices } = data;
        this.order = order;
        this.spentQuantity = spentQuantity;
        this.inputIndices = inputIndices;
        this.outputIndices = outputIndices;
    }

    /**
     * Convert to an object for RLP encoding.
     */
    public toEncodeObject() {
        const { order, spentQuantity, inputIndices, outputIndices } = this;
        return [
            order.toEncodeObject(),
            spentQuantity.toEncodeObject(),
            inputIndices,
            outputIndices
        ];
    }

    /**
     * Convert to RLP bytes.
     */
    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    /**
     * Convert to an OrderOnTransferJSON object.
     * @returns An OrderOnTransferJSON object.
     */
    public toJSON(): OrderOnTransferJSON {
        const { order, spentQuantity, inputIndices, outputIndices } = this;
        return {
            order: order.toJSON(),
            spentQuantity: spentQuantity.toJSON(),
            inputIndices,
            outputIndices
        };
    }

    /**
     * Return a consumed order as the spentQuantity.
     * @returns An Order object.
     */
    public getConsumedOrder(): Order {
        return this.order.consume(this.spentQuantity);
    }
}
