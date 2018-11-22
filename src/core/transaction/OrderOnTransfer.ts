import { U64 } from "../U64";

import { Order, OrderJSON } from "./Order";

const RLP = require("rlp");

export interface OrderOnTransferJSON {
    order: OrderJSON;
    spentAmount: string;
    inputIndices: number[];
    outputIndices: number[];
}

export interface OrderOnTransferData {
    order: Order;
    spentAmount: U64;
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
        const { order, spentAmount, inputIndices, outputIndices } = data;
        return new this({
            order: Order.fromJSON(order),
            spentAmount: U64.ensure(spentAmount),
            inputIndices,
            outputIndices
        });
    }

    public readonly order: Order;
    public readonly spentAmount: U64;
    public inputIndices: number[];
    public outputIndices: number[];

    /**
     * @param params.order An order to apply to the transfer transaction.
     * @param data.spentAmount A spent amount of the asset to give(from) while transferring.
     * @param data.inputIndices The indices of inputs affected by the order
     * @param data.outputIndices The indices of outputs affected by the order
     */
    constructor(data: OrderOnTransferData) {
        const { order, spentAmount, inputIndices, outputIndices } = data;
        this.order = order;
        this.spentAmount = spentAmount;
        this.inputIndices = inputIndices;
        this.outputIndices = outputIndices;
    }

    /**
     * Convert to an object for RLP encoding.
     */
    public toEncodeObject() {
        const { order, spentAmount, inputIndices, outputIndices } = this;
        return [
            order.toEncodeObject(),
            spentAmount.toEncodeObject(),
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
        const { order, spentAmount, inputIndices, outputIndices } = this;
        return {
            order: order.toJSON(),
            spentAmount: spentAmount.toString(),
            inputIndices,
            outputIndices
        };
    }

    /**
     * Return a consumed order as the spentAmount.
     * @returns An Order object.
     */
    public getConsumedOrder(): Order {
        return this.order.consume(this.spentAmount);
    }
}
