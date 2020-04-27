import { BigNumber } from "bignumber.js";
import { toLocaleString } from "../utility";

export type U64Value = U64 | BigNumber | number | string;

/**
 * @hidden
 */
const RLP = require("rlp");

/**
 * Handles 64-bit unsigned integers. Used to express nonce, asset amount, etc.
 */
export class U64 {
    public static MAX_VALUE = new U64(new BigNumber("0xFFFFFFFFFFFFFFFF"));

    public static plus(lhsValue: U64Value, rhsValue: U64Value): U64 {
        const lhs = U64.ensure(lhsValue);
        const rhs = U64.ensure(rhsValue);
        const result = lhs.value.plus(rhs.value);
        if (result.isGreaterThan(U64.MAX_VALUE.value)) {
            throw Error(`Integer overflow`);
        }
        return new U64(result);
    }

    public static minus(lhsValue: U64Value, rhsValue: U64Value): U64 {
        const lhs = U64.ensure(lhsValue);
        const rhs = U64.ensure(rhsValue);
        if (lhs.isLessThan(rhs)) {
            throw Error(`Integer underflow`);
        }
        return new U64(lhs.value.minus(rhs.value));
    }

    public static times(lhsValue: U64Value, rhsValue: U64Value): U64 {
        const lhs = U64.ensure(lhsValue);
        const rhs = U64.ensure(rhsValue);
        const result = lhs.value.times(rhs.value);
        if (result.isGreaterThan(U64.MAX_VALUE.value)) {
            throw Error(`Integer overflow`);
        }
        return new U64(result);
    }

    public static idiv(lhsValue: U64Value, rhsValue: U64Value): U64 {
        const lhs = U64.ensure(lhsValue);
        const rhs = U64.ensure(rhsValue);
        if (rhs.isEqualTo(0)) {
            throw Error(`Divided by 0`);
        }
        return new U64(lhs.value.idiv(rhs.value));
    }

    public static mod(lhsValue: U64Value, rhsValue: U64Value): U64 {
        const lhs = U64.ensure(lhsValue);
        const rhs = U64.ensure(rhsValue);
        if (rhs.isEqualTo(0)) {
            throw Error(`Divided by 0`);
        }
        return new U64(lhs.value.mod(rhs.value));
    }

    public static fromBytes(buffer: Buffer): U64 {
        const bytes = Array.from(buffer.values());
        const first = bytes.shift()!;
        if (first < 0x80) {
            return new U64(first);
        }
        const length = first! - 0x80;
        if (bytes.length !== length) {
            throw Error(`Invalid RLP for U64: ${bytes}`);
        } else if (length > 8) {
            throw Error("Buffer for U64 must be less than or equal to 8");
        } else if (length === 0) {
            return new U64(0);
        }
        return new U64(
            "0x" +
                bytes
                    .map(byte =>
                        byte < 0x10
                            ? `0${byte.toString(16)}`
                            : byte.toString(16)
                    )
                    .join("")
        );
    }

    public static check(param: any) {
        if (param instanceof U64) {
            return true;
        } else if (param instanceof BigNumber) {
            return (
                param.isInteger() &&
                !param.isNegative() &&
                param.isLessThanOrEqualTo(U64.MAX_VALUE.value)
            );
        } else if (typeof param === "number") {
            return Number.isInteger(param) && param >= 0;
        } else {
            return U64.checkString(param);
        }
    }

    public static ensure(param: U64Value) {
        return param instanceof U64 ? param : new U64(param);
    }

    private static checkString(param: string): boolean {
        if (typeof param !== "string") {
            return false;
        }
        const num = new BigNumber(param);
        return (
            num.isInteger() &&
            !num.isNegative() &&
            num.isLessThanOrEqualTo(U64.MAX_VALUE.value)
        );
    }

    public readonly value: BigNumber;

    constructor(value: number | string | BigNumber) {
        this.value = new BigNumber(value);
        if (!this.value.isInteger() || this.value.isNegative()) {
            throw Error(`U64 must be a positive integer but found ${value}`);
        } else if (this.value.toString(16).length > 16) {
            throw Error(
                `Given value is out of range for U64: ${this.value.toString(
                    16
                )}`
            );
        }
    }

    public plus(rhsValue: U64Value): U64 {
        return U64.plus(this, rhsValue);
    }

    public minus(rhsValue: U64Value): U64 {
        return U64.minus(this, rhsValue);
    }

    public times(rhsValue: U64Value): U64 {
        return U64.times(this, rhsValue);
    }

    public idiv(rhsValue: U64Value): U64 {
        return U64.idiv(this, rhsValue);
    }

    public mod(rhsValue: U64Value): U64 {
        return U64.mod(this, rhsValue);
    }

    public toEncodeObject(): string | number {
        const hex = this.value.toString(16);
        // NOTE: workaround that RLP.encode("0x0") results to 00
        if (hex === "0") {
            return 0;
        } else {
            return hex.length % 2 === 0 ? `0x${hex}` : `0x0${hex}`;
        }
    }

    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    public isEqualTo(rhs: U64Value): boolean {
        return this.value.isEqualTo(U64.ensure(rhs).value);
    }

    public eq(rhs: U64Value): boolean {
        return this.isEqualTo(rhs);
    }

    public isGreaterThan(rhs: U64Value): boolean {
        return this.value.isGreaterThan(U64.ensure(rhs).value);
    }

    public gt(rhs: U64Value): boolean {
        return this.isGreaterThan(rhs);
    }

    public isGreaterThanOrEqualTo(rhs: U64Value): boolean {
        return this.value.isGreaterThanOrEqualTo(U64.ensure(rhs).value);
    }

    public gte(rhs: U64Value): boolean {
        return this.isGreaterThanOrEqualTo(rhs);
    }

    public isLessThan(rhs: U64Value): boolean {
        return this.value.isLessThan(U64.ensure(rhs).value);
    }

    public lt(rhs: U64Value): boolean {
        return this.isLessThan(rhs);
    }

    public isLessThanOrEqualTo(rhs: U64Value): boolean {
        return this.value.isLessThanOrEqualTo(U64.ensure(rhs).value);
    }

    public lte(rhs: U64Value): boolean {
        return this.isLessThanOrEqualTo(rhs);
    }

    public toString(base?: 10 | 16) {
        return this.value.toString(base || 10);
    }

    public toLocaleString() {
        return toLocaleString(this.value);
    }

    public toJSON(): string {
        return `0x${this.value.toString(16)}`;
    }
}
