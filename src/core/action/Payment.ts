import { PlatformAddress } from "codechain-primitives";

import { U256 } from "../U256";

export class Payment {
    public receiver: PlatformAddress;
    public amount: U256;

    constructor(receiver: PlatformAddress, amount: U256) {
        this.receiver = receiver;
        this.amount = amount;
    }

    public toEncodeObject(): any[] {
        return [
            2,
            this.receiver.getAccountId().toEncodeObject(),
            this.amount.toEncodeObject()
        ];
    }

    public toJSON() {
        return {
            action: "payment",
            receiver: this.receiver.value,
            amount: this.amount.value.toString()
        };
    }
}
