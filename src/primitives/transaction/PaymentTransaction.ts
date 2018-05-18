import { H160, U256 } from "../index";

export type PaymentTransactionData = {
    address: H160;
    value: U256;
};

export class PaymentTransaction {
    private data: PaymentTransactionData;

    constructor(data: PaymentTransactionData) {
        this.data = data;
    }

    toEncodeObject() {
        const { address, value } = this.data;
        return [0x01, address.toEncodeObject(), value.toEncodeObject()];
    }
}