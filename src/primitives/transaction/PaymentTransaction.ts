import { H160 } from "../H160";
import { H256 } from "../H256";
import { U256 } from "../U256";
import { blake256 } from "../../utils";

const RLP = require("rlp");

export type PaymentTransactionData = {
    nonce: U256;
    sender: H160;
    receiver: H160;
    value: U256;
};

/**
 * Sends value amount of CCC from sender to receiver.
 *
 * - The nonce has to be identical with the sender account's nonce.
 * - The transaction fails if the nonce is not identical, or if the sender does not have value amount of CCC.
 */
export class PaymentTransaction {
    private data: PaymentTransactionData;
    private type = "payment";

    constructor(data: PaymentTransactionData) {
        this.data = data;
    }

    toEncodeObject() {
        const { sender, receiver, nonce, value } = this.data;
        return [0x01, nonce.toEncodeObject(), sender.toEncodeObject(), receiver.toEncodeObject(), value.toEncodeObject()];
    }

    static fromJSON(data: any) {
        const { nonce, sender, receiver, value } = data["payment"];
        return new PaymentTransaction({
            nonce: new U256(nonce),
            sender: new H160(sender),
            receiver: new H160(receiver),
            value: new U256(value),
        });
    }

    toJSON() {
        const { nonce, sender, receiver, value } = this.data;
        return {
            [this.type]: {
                nonce: nonce.value.toString(),
                sender: sender.value,
                receiver: receiver.value,
                value: value.value.toString(),
            }
        };
    }

    rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }
}
