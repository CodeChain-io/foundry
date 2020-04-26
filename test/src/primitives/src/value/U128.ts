import { BigNumber } from "bignumber.js";
import { toLocaleString } from "../utility";

import { U64, U64Value } from "./U64";

export type U128Value = U128 | U64Value;

/**
 * @hidden
 */
const RLP = require("rlp");

/**
 * Handles 128-bit unsigned integers. Used to express nonce, asset amount, etc.
 */
export class U128 {
    public static MAX_VALUE = new U128(
        new BigNumber("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF")
    );

    public static plus(lhsValue: U128Value, rhsValue: U128Value): U128 {
        const lhs = U128.ensure(lhsValue);
        const rhs = U128.ensure(rhsValue);
        const result = lhs.value.plus(rhs.value);
        if (result.isGreaterThan(U128.MAX_VALUE.value)) {
            throw Error(`Integer overflow`);
        }
        return new U128(result);
    }

    public static minus(lhsValue: U128Value, rhsValue: U128Value): U128 {
        const lhs = U128.ensure(lhsValue);
        const rhs = U128.ensure(rhsValue);
        if (lhs.isLessThan(rhs)) {
            throw Error(`Integer underflow`);
        }
        return new U128(lhs.value.minus(rhs.value));
    }

    public static times(lhsValue: U128Value, rhsValue: U128Value): U128 {
        const lhs = U128.ensure(lhsValue);
        const rhs = U128.ensure(rhsValue);
        const result = lhs.value.times(rhs.value);
        if (result.isGreaterThan(U128.MAX_VALUE.value)) {
            throw Error(`Integer overflow`);
        }
        return new U128(result);
    }

    public static idiv(lhsValue: U128Value, rhsValue: U128Value): U128 {
        const lhs = U128.ensure(lhsValue);
        const rhs = U128.ensure(rhsValue);
        if (rhs.isEqualTo(0)) {
            throw Error(`Divided by 0`);
        }
        return new U128(lhs.value.idiv(rhs.value));
    }

    public static mod(lhsValue: U128Value, rhsValue: U128Value): U128 {
        const lhs = U128.ensure(lhsValue);
        const rhs = U128.ensure(rhsValue);
        if (rhs.isEqualTo(0)) {
            throw Error(`Divided by 0`);
        }
        return new U128(lhs.value.mod(rhs.value));
    }

    public static fromBytes(buffer: Buffer): U128 {
        const bytes = Array.from(buffer.values());
        const first = bytes.shift()!;
        if (first < 0x80) {
            return new U64(first);
        }
        const length = first - 0x80;
        if (bytes.length !== length) {
            throw Error(`Invalid RLP for U128: ${bytes}`);
        } else if (length > 16) {
            throw Error("Buffer for U128 must be less than or equal to 16");
        } else if (length === 0) {
            return new U128(0);
        }
        return new U128(
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
        if (param instanceof U128 || param instanceof U64) {
            return true;
        } else if (param instanceof BigNumber) {
            return (
                param.isInteger() &&
                !param.isNegative() &&
                param.isLessThanOrEqualTo(U128.MAX_VALUE.value)
            );
        } else if (typeof param === "number") {
            return Number.isInteger(param) && param >= 0;
        } else {
            return U128.checkString(param);
        }
    }

    public static ensure(param: U128Value) {
        return param instanceof U128 ? param : new U128(param);
    }

    private static checkString(param: string): boolean {
        if (typeof param !== "string") {
            return false;
        }
        const num = new BigNumber(param);
        return (
            num.isInteger() &&
            !num.isNegative() &&
            num.isLessThanOrEqualTo(U128.MAX_VALUE.value)
        );
    }

    public readonly value: BigNumber;

    constructor(value: number | string | BigNumber | U64) {
        this.value = new BigNumber(value instanceof U64 ? value.value : value);
        if (!this.value.isInteger() || this.value.isNegative()) {
            throw Error(`U128 must be a positive integer but found ${value}`);
        } else if (this.value.toString(16).length > 32) {
            throw Error(
                `Given value is out of range for U128: ${this.value.toString(
                    16
                )}`
            );
        }
    }

    public plus(rhsValue: U128Value): U128 {
        return U128.plus(this, rhsValue);
    }

    public minus(rhsValue: U128Value): U128 {
        return U128.minus(this, rhsValue);
    }

    public times(rhsValue: U128Value): U128 {
        return U128.times(this, rhsValue);
    }

    public idiv(rhsValue: U128Value): U128 {
        return U128.idiv(this, rhsValue);
    }

    public mod(rhsValue: U128Value): U128 {
        return U128.mod(this, rhsValue);
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

    public isEqualTo(rhs: U128Value): boolean {
        return this.value.isEqualTo(U128.ensure(rhs).value);
    }

    public eq(rhs: U128Value): boolean {
        return this.isEqualTo(rhs);
    }

    public isGreaterThan(rhs: U128Value): boolean {
        return this.value.isGreaterThan(U128.ensure(rhs).value);
    }

    public gt(rhs: U128Value): boolean {
        return this.isGreaterThan(rhs);
    }

    public isGreaterThanOrEqualTo(rhs: U128Value): boolean {
        return this.value.isGreaterThanOrEqualTo(U128.ensure(rhs).value);
    }

    public gte(rhs: U128Value): boolean {
        return this.isGreaterThanOrEqualTo(rhs);
    }

    public isLessThan(rhs: U128Value): boolean {
        return this.value.isLessThan(U128.ensure(rhs).value);
    }

    public lt(rhs: U128Value): boolean {
        return this.isLessThan(rhs);
    }

    public isLessThanOrEqualTo(rhs: U128Value): boolean {
        return this.value.isLessThanOrEqualTo(U128.ensure(rhs).value);
    }

    public lte(rhs: U128Value): boolean {
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
