/**
 * @hidden
 */
const RLP = require("rlp");

export type H512Value = H512 | string;

/**
 * Handles 512-bit data. Used to express public keys.
 */
export class H512 {
    public static fromBytes(buffer: Buffer): H512 {
        const bytes = Array.from(buffer.values());
        const firstByte = bytes.shift();
        const length = bytes.shift();
        if (firstByte !== 0xb8 || length !== 64 || bytes.length !== length) {
            throw Error(`Invalid RLP for H512: ${bytes}`);
        }
        return new H512(
            bytes
                .map(byte =>
                    byte < 0x10 ? `0${byte.toString(16)}` : byte.toString(16)
                )
                .join("")
        );
    }

    public static zero(): H512 {
        return new H512(
            "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    public static check(param: any): boolean {
        return param instanceof H512 ? true : H512.checkString(param);
    }

    public static ensure(param: H512Value): H512 {
        return param instanceof H512 ? param : new H512(param);
    }

    private static checkString(value: string): boolean {
        return /^(0x)?[0-9a-fA-F]{128}$/.test(value);
    }

    public value: string;

    constructor(value: string) {
        if (!H512.checkString(value)) {
            throw Error(
                `Expected 64 byte hexstring for creating H512 but found "${value}"`
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

    public isEqualTo(rhs: H512): boolean {
        return this.value === rhs.value;
    }

    public toString(): string {
        return this.value;
    }

    public toJSON() {
        return `0x${this.value}`;
    }
}
