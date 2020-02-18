import { U64 } from "codechain-primitives";

import { Order, OrderJSON } from "./Order";

const RLP = require("rlp");

export interface OrderOnTransferJSON {
    order: OrderJSON;
    spentQuantity: string;
    inputFromIndices: number[];
    inputFeeIndices: number[];
    outputFromIndices: number[];
    outputToIndices: number[];
    outputOwnedFeeIndices: number[];
    outputTransferredFeeIndices: number[];
}

export interface OrderOnTransferData {
    order: Order;
    spentQuantity: U64;
    inputFromIndices: number[];
    inputFeeIndices: number[];
    outputFromIndices: number[];
    outputToIndices: number[];
    outputOwnedFeeIndices: number[];
    outputTransferredFeeIndices: number[];
}

export class OrderOnTransfer {
    /**
     * Create an Order from an OrderJSON object.
     * @param data An OrderJSON object.
     * @returns An Order.
     */
    public static fromJSON(data: OrderOnTransferJSON) {
        const {
            order,
            spentQuantity,
            inputFromIndices,
            inputFeeIndices,
            outputFromIndices,
            outputToIndices,
            outputOwnedFeeIndices,
            outputTransferredFeeIndices
        } = data;
        return new OrderOnTransfer({
            order: Order.fromJSON(order),
            spentQuantity: U64.ensure(spentQuantity),
            inputFromIndices,
            inputFeeIndices,
            outputFromIndices,
            outputToIndices,
            outputOwnedFeeIndices,
            outputTransferredFeeIndices
        });
    }

    public readonly order: Order;
    public readonly spentQuantity: U64;
    public inputFromIndices: number[];
    public inputFeeIndices: number[];
    public outputFromIndices: number[];
    public outputToIndices: number[];
    public outputOwnedFeeIndices: number[];
    public outputTransferredFeeIndices: number[];

    /**
     * @param params.order An order to apply to the transfer transaction.
     * @param data.spentQuantity A spent quantity of the asset to give(from) while transferring.
     * @param data.inputIndices The indices of inputs affected by the order
     * @param data.outputIndices The indices of outputs affected by the order
     */
    constructor(data: OrderOnTransferData) {
        const {
            order,
            spentQuantity,
            inputFromIndices,
            inputFeeIndices,
            outputFromIndices,
            outputToIndices,
            outputOwnedFeeIndices,
            outputTransferredFeeIndices
        } = data;
        this.order = order;
        this.spentQuantity = spentQuantity;
        this.inputFromIndices = inputFromIndices;
        this.inputFeeIndices = inputFeeIndices;
        this.outputFromIndices = outputFromIndices;
        this.outputToIndices = outputToIndices;
        this.outputOwnedFeeIndices = outputOwnedFeeIndices;
        this.outputTransferredFeeIndices = outputTransferredFeeIndices;
    }

    /**
     * Convert to an object for RLP encoding.
     */
    public toEncodeObject() {
        const {
            order,
            spentQuantity,
            inputFromIndices,
            inputFeeIndices,
            outputFromIndices,
            outputToIndices,
            outputOwnedFeeIndices,
            outputTransferredFeeIndices
        } = this;
        return [
            order.toEncodeObject(),
            spentQuantity.toEncodeObject(),
            inputFromIndices,
            inputFeeIndices,
            outputFromIndices,
            outputToIndices,
            outputOwnedFeeIndices,
            outputTransferredFeeIndices
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
        const {
            order,
            spentQuantity,
            inputFromIndices,
            inputFeeIndices,
            outputFromIndices,
            outputToIndices,
            outputOwnedFeeIndices,
            outputTransferredFeeIndices
        } = this;
        return {
            order: order.toJSON(),
            spentQuantity: spentQuantity.toJSON(),
            inputFromIndices,
            inputFeeIndices,
            outputFromIndices,
            outputToIndices,
            outputOwnedFeeIndices,
            outputTransferredFeeIndices
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
