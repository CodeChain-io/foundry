import * as _ from "lodash";
import { BigNumber } from "bignumber.js";

const RLP = require("rlp");

export class U256 {
    value: BigNumber;

    constructor(value?: number | string) {
        this.value = new BigNumber(value || 0);
        if (!this.value.isInteger() || this.value.isNegative()) {
            throw "U256 must be a positive integer";
        } else if (this.value.toString(16).length > 64) {
            throw "Given value is out of range for U256";
        }
    }

    static fromBytes(buffer: Buffer): U256 {
        const bytes = Array.from(buffer.values());
        const length = bytes.shift()! - 0x80;
        if (length > 32) {
            throw "Buffer for U256 must be less than or equal to 32";
        } else if (bytes.length !== length) {
            throw "Invalid RLP";
        } else if (length === 0) {
            return new U256(0);
        }
        return new U256("0x" + bytes.map(byte => byte < 0x10 ? `0${byte.toString(16)}` : byte.toString(16)).join(""));
    }

    toEncodeObject(): string | number {
        const hex = this.value.toString(16);
        // NOTE: workaround that RLP.encode("0x0") results to 00
        if (hex === "0") {
            return 0;
        } else {
            return hex.length % 2 === 0
                ? `0x${hex}`
                : `0x0${hex}`;
        }
    }

    rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }
}
