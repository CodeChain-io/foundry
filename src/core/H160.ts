import * as _ from "lodash";

const RLP = require("rlp");
/**
 * Handles 160-bit data. Used to express account addresses.
 */
export class H160 {
    public static fromBytes(buffer: Buffer): H160 {
        const bytes = Array.from(buffer.values());
        const length = bytes.shift()! - 0x80;
        if (length !== 20 || bytes.length !== length) {
            throw Error(`Invalid RLP for H160: ${bytes}`);
        }
        return new H160(
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

    public static check(param: H160 | string): boolean {
        return param instanceof H160 ? true : H160.checkString(param);
    }

    public static ensure(param: H160 | string): H160 {
        return param instanceof H160 ? param : new H160(param);
    }

    private static checkString(value: string): boolean {
        return /^(0x)?[0-9a-fA-F]{40}$/.test(value);
    }

    public value: string;

    constructor(value: string) {
        if (!H160.checkString(value)) {
            throw Error(
                `Expected 20 byte hexstring for creating H160 but found "${value}"`
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
}
