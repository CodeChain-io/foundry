import { H160, H256, U256 } from "../index";
import { blake256 } from "../../utils";

const RLP = require("rlp");

export type PaymentTransactionData = {
    nonce: U256;
    address: H160;
    value: U256;
};

export class PaymentTransaction {
    private data: PaymentTransactionData;
    private type = "payment";

    constructor(data: PaymentTransactionData) {
        this.data = data;
    }

    toEncodeObject() {
        const { address, nonce, value } = this.data;
        return [0x01, nonce.toEncodeObject(), address.toEncodeObject(), value.toEncodeObject()];
    }

    static fromJSON(data: any) {
        const { nonce, address, value } = data["payment"];
        return new PaymentTransaction({
            nonce: new U256(nonce),
            address: new H160(address),
            value: new U256(value),
        });
    }

    toJSON() {
        const { nonce, address, value } = this.data;
        return {
            [this.type]: {
                nonce: nonce.value.toString(),
                address: address.value,
                value: value.value.toString(),
            }
        };
    }

    rlpBytes() {
        return RLP.encode(this.toEncodeObject());
    }

    hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }
}
