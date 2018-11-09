"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
class Payment {
    constructor(receiver, amount) {
        this.receiver = receiver;
        this.amount = amount;
    }
    toEncodeObject() {
        return [
            2,
            this.receiver.getAccountId().toEncodeObject(),
            this.amount.toEncodeObject()
        ];
    }
    toJSON() {
        return {
            action: "payment",
            receiver: this.receiver.value,
            amount: this.amount.toEncodeObject()
        };
    }
}
exports.Payment = Payment;
