const RLP = require("rlp");
/**
 * Handles 256-bit data. Used to express block hash, parcel hash, transaction hash, merkle root, etc.
 */
export class H256 {
    value: string;

    constructor(value: string) {
        if ((!value.startsWith("0x") && value.length !== 64) || (value.startsWith("0x") && value.length !== 66)) {
            throw `The length for H256 must be 64 or 66 with 0x-prefix, but "${value}" is given`;
        } else if (!/(0x)?[0-9a-fA-F]{64}/.test(value)) {
            throw `Invalid hexadecimal string: ${value}`;
        }
        this.value = value.startsWith("0x") ? value.slice(2).toLowerCase() : value.toLowerCase();
    }

    static fromBytes(buffer: Buffer): H256 {
        const bytes = Array.from(buffer.values());
        const length = bytes.shift()! - 0x80;
        if (length !== 32 || bytes.length !== length) {
            throw `Invalid RLP for H256: ${bytes}`;
        }
        return new H256(bytes.map(byte => byte < 0x10 ? `0${byte.toString(16)}` : byte.toString(16)).join(""));
    }

    static ensure(param: H256 | string): H256 {
        return param instanceof H256 ? param : new H256(param);
    }

    toEncodeObject(): string {
        return `0x${this.value}`;
    }

    rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    isEqualTo(rhs: H256): boolean {
        return this.value === rhs.value;
    }
}
