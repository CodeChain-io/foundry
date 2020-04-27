/**
 * @hidden
 */
const RLP = require("rlp");

export type H128Value = H128 | string;

/**
 * Handles 128-bit data.
 */
export class H128 {
    public static fromBytes(buffer: Buffer): H128 {
        const bytes = Array.from(buffer.values());
        const length = bytes.shift()! - 0x80;
        if (length !== 16 || bytes.length !== length) {
            throw Error(`Invalid RLP for H128: ${bytes}`);
        }
        return new H128(
            bytes
                .map(byte =>
                    byte < 0x10 ? `0${byte.toString(16)}` : byte.toString(16)
                )
                .join("")
        );
    }

    public static zero(): H128 {
        return new H128("00000000000000000000000000000000");
    }

    public static check(param: any): boolean {
        return param instanceof H128 ? true : H128.checkString(param);
    }

    public static ensure(param: H128Value): H128 {
        return param instanceof H128 ? param : new H128(param);
    }

    private static checkString(value: string): boolean {
        return /^(0x)?[0-9a-fA-F]{32}$/.test(value);
    }

    public value: string;

    constructor(value: string) {
        if (!H128.checkString(value)) {
            throw Error(
                `Expected 16 byte hexstring for creating H128 but found "${value}"`
            );
        }
        this.value = value.startsWith("0x")
            ? value.slice(2).toLowerCase()
            : value.toLowerCase();
    }

    public toEncodeObject(): string {
        return `0x${this.value}`;
    }

    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    public isEqualTo(rhs: H128): boolean {
        return this.value === rhs.value;
    }

    public toString(): string {
        return this.value;
    }

    public toJSON() {
        return `0x${this.value}`;
    }
}
