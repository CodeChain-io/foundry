import { PlatformAddress } from "../../key/PlatformAddress";
import { U256 } from "../U256";

export class Payment {
    receiver: PlatformAddress;
    amount: U256;

    constructor(receiver: PlatformAddress, amount: U256) {
        this.receiver = receiver;
        this.amount = amount;
    }

    toEncodeObject(): Array<any> {
        return [2, this.receiver.getAccountId().toEncodeObject(), this.amount.toEncodeObject()];
    }

    toJSON() {
        return {
            action: "payment",
            receiver: this.receiver.value,
            amount: this.amount.value.toString()
        };
    }
}
