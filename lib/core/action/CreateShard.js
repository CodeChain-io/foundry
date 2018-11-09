"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
class CreateShard {
    toEncodeObject() {
        return [4];
    }
    toJSON() {
        return {
            action: "createShard"
        };
    }
}
exports.CreateShard = CreateShard;
