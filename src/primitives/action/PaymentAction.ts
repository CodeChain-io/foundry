import { H160, U256 } from "../index";

export type PaymentActionData = {
    address: H160;
    value: U256;
};

export class PaymentAction {
    private data: PaymentActionData;

    constructor(data: PaymentActionData) {
        this.data = data;
    }

    toEncodeObject() {
        const { address, value } = this.data;
        return [0x01, address.toEncodeObject(), value.toEncodeObject()];
    }
}