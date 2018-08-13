const RLP = require("rlp");
/**
 * Handles 160-bit data. Used to express account addresses.
 */
export class H160 {
    value: string;

    constructor(value: string) {
        if ((!value.startsWith("0x") && value.length !== 40) || (value.startsWith("0x") && value.length !== 42)) {
            throw `The length for H160 must be 40 or 42 with 0x-prefix, but "${value}" is given`;
        } else if (!/(0x)?[0-9a-fA-F]{40}/.test(value)) {
            throw `Invalid hexadecimal string: ${value}`;
        }
        this.value = value.startsWith("0x") ? value.slice(2).toLowerCase() : value.toLowerCase();
    }

    static fromBytes(buffer: Buffer): H160 {
        const bytes = Array.from(buffer.values());
        const length = bytes.shift()! - 0x80;
        if (length !== 20 || bytes.length !== length) {
            throw `Invalid RLP for H160: ${bytes}`;
        }
        return new H160(bytes.map(byte => byte < 0x10 ? `0${byte.toString(16)}` : byte.toString(16)).join(""));
    }

    static ensure(param: H160 | string): H160 {
        return param instanceof H160 ? param : new H160(param);
    }

    toEncodeObject(): string {
        return `0x${this.value}`;
    }

    rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }
}
