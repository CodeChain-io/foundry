import { BigNumber } from "bignumber.js";

import { toLocaleString } from "../utility";
import { U128, U128Value } from "./U128";
import { U64 } from "./U64";

export type U256Value = U256 | U128Value;

/**
 * @hidden
 */
const RLP = require("rlp");

/**
 * Handles 256-bit unsigned integers.
 */
export class U256 {
    public static MAX_VALUE = new U256(
        new BigNumber(
            "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
        )
    );

    public static plus(lhsValue: U256Value, rhsValue: U256Value): U256 {
        const lhs = U256.ensure(lhsValue);
        const rhs = U256.ensure(rhsValue);
        const result = lhs.value.plus(rhs.value);
        if (result.isGreaterThan(U256.MAX_VALUE.value)) {
            throw Error(`Integer overflow`);
        }
        return new U256(result);
    }

    public static minus(lhsValue: U256Value, rhsValue: U256Value): U256 {
        const lhs = U256.ensure(lhsValue);
        const rhs = U256.ensure(rhsValue);
        if (lhs.isLessThan(rhs)) {
            throw Error(`Integer underflow`);
        }
        return new U256(lhs.value.minus(rhs.value));
    }

    public static times(lhsValue: U256Value, rhsValue: U256Value): U256 {
        const lhs = U256.ensure(lhsValue);
        const rhs = U256.ensure(rhsValue);
        const result = lhs.value.times(rhs.value);
        if (result.isGreaterThan(U256.MAX_VALUE.value)) {
            throw Error(`Integer overflow`);
        }
        return new U256(result);
    }

    public static idiv(lhsValue: U256Value, rhsValue: U256Value): U256 {
        const lhs = U256.ensure(lhsValue);
        const rhs = U256.ensure(rhsValue);
        if (rhs.isEqualTo(0)) {
            throw Error(`Divided by 0`);
        }
        return new U256(lhs.value.idiv(rhs.value));
    }

    public static mod(lhsValue: U256Value, rhsValue: U256Value): U256 {
        const lhs = U256.ensure(lhsValue);
        const rhs = U256.ensure(rhsValue);
        if (rhs.isEqualTo(0)) {
            throw Error(`Divided by 0`);
        }
        return new U256(lhs.value.mod(rhs.value));
    }

    public static fromBytes(buffer: Buffer): U256 {
        const bytes = Array.from(buffer.values());
        const first = bytes.shift()!;
        if (first < 0x80) {
            return new U64(first);
        }
        const length = first! - 0x80;
        if (bytes.length !== length) {
            throw Error(`Invalid RLP for U256: ${bytes}`);
        } else if (length > 32) {
            throw Error("Buffer for U256 must be less than or equal to 32");
        } else if (length === 0) {
            return new U256(0);
        }
        return new U256(
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

    public static check(param: any): boolean {
        if (
            param instanceof U256 ||
            param instanceof U128 ||
            param instanceof U64
        ) {
            return true;
        } else if (param instanceof BigNumber) {
            return (
                param.isInteger() &&
                !param.isNegative() &&
                param.isLessThanOrEqualTo(U256.MAX_VALUE.value)
            );
        } else if (typeof param === "number") {
            return Number.isInteger(param) && param >= 0;
        } else {
            return U256.checkString(param);
        }
    }

    public static ensure(param: U256Value): U256 {
        return param instanceof U256
            ? param
            : new U256(
                  param instanceof U128 || param instanceof U64
                      ? param.value
                      : param
              );
    }

    private static checkString(param: string): boolean {
        if (typeof param !== "string") {
            return false;
        }
        const num = new BigNumber(param);
        return (
            num.isInteger() &&
            !num.isNegative() &&
            num.isLessThanOrEqualTo(U256.MAX_VALUE.value)
        );
    }

    public value: BigNumber;

    constructor(value: number | string | BigNumber | U128 | U64) {
        this.value = new BigNumber(
            value instanceof U128 || value instanceof U64 ? value.value : value
        );
        if (!this.value.isInteger() || this.value.isNegative()) {
            throw Error(`U256 must be a positive integer but found ${value}`);
        } else if (this.value.toString(16).length > 64) {
            throw Error(
                `Given value is out of range for U256: ${this.value.toString(
                    16
                )}`
            );
        }
    }

    public plus(rhsValue: U256Value): U256 {
        return U256.plus(this, rhsValue);
    }

    public minus(rhsValue: U256Value): U256 {
        return U256.minus(this, rhsValue);
    }

    public times(rhsValue: U256Value): U256 {
        return U256.times(this, rhsValue);
    }

    public idiv(rhsValue: U256Value): U256 {
        return U256.idiv(this, rhsValue);
    }

    public mod(rhsValue: U256Value): U256 {
        return U256.mod(this, rhsValue);
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

    public isEqualTo(rhs: U256Value): boolean {
        return this.value.isEqualTo(U256.ensure(rhs).value);
    }

    public eq(rhs: U256Value): boolean {
        return this.isEqualTo(rhs);
    }

    public isGreaterThan(rhs: U256Value): boolean {
        return this.value.isGreaterThan(U256.ensure(rhs).value);
    }

    public gt(rhs: U256Value): boolean {
        return this.isGreaterThan(rhs);
    }

    public isGreaterThanOrEqualTo(rhs: U256Value): boolean {
        return this.value.isGreaterThanOrEqualTo(U256.ensure(rhs).value);
    }

    public gte(rhs: U256Value): boolean {
        return this.isGreaterThanOrEqualTo(rhs);
    }

    public isLessThan(rhs: U256Value): boolean {
        return this.value.isLessThan(U256.ensure(rhs).value);
    }

    public lt(rhs: U256Value): boolean {
        return this.isLessThan(rhs);
    }

    public isLessThanOrEqualTo(rhs: U256Value): boolean {
        return this.value.isLessThanOrEqualTo(U256.ensure(rhs).value);
    }

    public lte(rhs: U256Value): boolean {
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
