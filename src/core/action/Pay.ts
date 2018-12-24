import { PlatformAddress } from "codechain-primitives";

import { U64 } from "../U64";

export class Pay {
    public receiver: PlatformAddress;
    public amount: U64;

    constructor(receiver: PlatformAddress, amount: U64) {
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
            action: "pay",
            receiver: this.receiver.value,
            amount: this.amount.toEncodeObject()
        };
    }
}
