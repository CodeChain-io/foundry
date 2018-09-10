const RLP = require("rlp");
/**
 * Handles 256-bit data. Used to express block hash, parcel hash, transaction hash, merkle root, etc.
 */
export class H256 {
    public static fromBytes(buffer: Buffer): H256 {
        const bytes = Array.from(buffer.values());
        const length = bytes.shift()! - 0x80;
        if (length !== 32 || bytes.length !== length) {
            throw Error(`Invalid RLP for H256: ${bytes}`);
        }
        return new H256(
            bytes
                .map(
                    byte =>
                        byte < 0x10
                            ? `0${byte.toString(16)}`
                            : byte.toString(16)
                )
                .join("")
        );
    }

    public static check(param: H256 | string): boolean {
        return param instanceof H256 ? true : H256.checkString(param);
    }

    public static ensure(param: H256 | string): H256 {
        return param instanceof H256 ? param : new H256(param);
    }

    private static checkString(value: string): boolean {
        return /^(0x)?[0-9a-fA-F]{64}$/.test(value);
    }

    public value: string;

    constructor(value: string) {
        if (!H256.checkString(value)) {
            throw Error(
                `Expected 32 byte hexstring for creating H256 but found "${value}"`
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

    public isEqualTo(rhs: H256): boolean {
        return this.value === rhs.value;
    }
}
