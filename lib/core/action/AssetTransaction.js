"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
class AssetTransaction {
    constructor(input) {
        this.transaction = input.transaction;
    }
    toEncodeObject() {
        const transaction = this.transaction.toEncodeObject();
        return [1, transaction];
    }
    toJSON() {
        return {
            action: "assetTransaction",
            transaction: this.transaction.toJSON()
        };
    }
}
exports.AssetTransaction = AssetTransaction;
