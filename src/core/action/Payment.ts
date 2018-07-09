import { H160 } from "../H160";
import { U256 } from "../U256";

export class Payment {
    receiver: H160;
    amount: U256;

    constructor(receiver: H160, amount: U256) {
        this.receiver = receiver;
        this.amount = amount;
    }

    toEncodeObject(): Array<any> {
        return [2, this.receiver.toEncodeObject(), this.amount.toEncodeObject()];
    }

    toJSON() {
        return {
            action: "payment",
            receiver: this.receiver.value,
            amount: this.amount.value.toString()
        };
    }
}
