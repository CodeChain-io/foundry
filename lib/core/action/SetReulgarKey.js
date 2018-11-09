"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
class SetRegularKey {
    constructor(key) {
        this.key = key;
    }
    toEncodeObject() {
        return [3, this.key.toEncodeObject()];
    }
    toJSON() {
        return {
            action: "setRegularKey",
            key: this.key.value
        };
    }
}
exports.SetRegularKey = SetRegularKey;
