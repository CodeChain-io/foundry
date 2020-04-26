import BigNumber from "bignumber.js";

import { U128, U256, U64 } from "..";

const TOO_LARGE =
    "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF";

describe.each([
    [U64, "U64", 8],
    [U128, "U128", 16],
    [U256, "U256", 32]
])("%p", (Uxxx, className, byteLength) => {
    test("import", () => {
        expect(typeof Uxxx).toBe("function");
    });

    test("require", () => {
        const obj = require("..");
        expect(typeof obj[className]).toBe("function");
    });

    test("new", () => {
        expect(new Uxxx(16).eq(new Uxxx("16"))).toBe(true);
        expect(new Uxxx(16).eq(new Uxxx("0x10"))).toBe(true);
        expect(new Uxxx(16).eq(new Uxxx(new BigNumber(16)))).toBe(true);
        expect(new Uxxx(16).eq(new Uxxx(new BigNumber("16")))).toBe(true);
        expect(new Uxxx(16).eq(new Uxxx(new BigNumber("0x10")))).toBe(true);

        expect(() => new Uxxx(TOO_LARGE)).toThrow();

        if (Uxxx === U256) {
            expect(new Uxxx(16).eq(new U256(new U64(16)))).toBe(true);
            expect(new Uxxx(16).eq(new U256(new U128(16)))).toBe(true);
        }
        if (Uxxx === U128) {
            expect(new Uxxx(16).eq(new U128(new U64(16)))).toBe(true);
        }
    });

    test("check", () => {
        expect(Uxxx.check(undefined as any)).toBe(false);
        expect(Uxxx.check(null as any)).toBe(false);
        expect(Uxxx.check(-1)).toBe(false);
        expect(Uxxx.check(0.5)).toBe(false);
        expect(Uxxx.check(0.5)).toBe(false);

        expect(Uxxx.check(new BigNumber(-1))).toBe(false);
        expect(Uxxx.check(new BigNumber(0.5))).toBe(false);
        expect(Uxxx.check(new BigNumber(TOO_LARGE))).toBe(false);

        expect(Uxxx.check(0)).toBe(true);
        expect(Uxxx.check("0")).toBe(true);
        expect(Uxxx.check("0x0")).toBe(true);

        if (byteLength >= 32) {
            expect(Uxxx.check(new U256(0))).toBe(true);
        }
        if (byteLength >= 16) {
            expect(Uxxx.check(new U128(0))).toBe(true);
        }
        if (byteLength >= 8) {
            expect(Uxxx.check(new U64(0))).toBe(true);
        }
    });

    test("ensure", () => {
        expect(() => {
            Uxxx.ensure(undefined as any);
        }).toThrow();
        expect(Uxxx.ensure(10)).toEqual(new Uxxx(10));
        expect(Uxxx.ensure("10")).toEqual(new Uxxx(10));
        expect(Uxxx.ensure("0xA")).toEqual(new Uxxx(10));
        expect(Uxxx.ensure(new Uxxx(10))).toEqual(new Uxxx(10));

        if (Uxxx === U256) {
            expect(Uxxx.ensure(new U64(10))).toEqual(new Uxxx(10));
        }
    });

    test("fromBytes", () => {
        let a;
        a = new Uxxx(0);
        expect(Uxxx.fromBytes(a.rlpBytes())).toEqual(a);
        a = new Uxxx(1);
        expect(Uxxx.fromBytes(a.rlpBytes())).toEqual(a);
        a = new Uxxx(0x79);
        expect(Uxxx.fromBytes(a.rlpBytes())).toEqual(a);
        a = new Uxxx(255);
        expect(Uxxx.fromBytes(a.rlpBytes())).toEqual(a);
        a = new Uxxx(1000);
        expect(Uxxx.fromBytes(a.rlpBytes())).toEqual(a);
        a = new Uxxx("1000000000000");
        expect(Uxxx.fromBytes(a.rlpBytes())).toEqual(a);
        const buf = new Buffer(byteLength + 1);
        buf[0] = 0x80 + byteLength;
        buf.fill(0xff, 1, byteLength + 1);
        expect(Uxxx.fromBytes(buf)).toEqual(Uxxx.MAX_VALUE);
    });

    test("fromBytes throws for oversize Buffer", () => {
        expect(() => {
            const buf = new Buffer(byteLength + 2);
            buf[0] = 0x80 + byteLength + 1;
            buf.fill(0xff, 1, byteLength + 2);
            expect(Uxxx.fromBytes(buf)).toEqual(Uxxx.MAX_VALUE);
        }).toThrow(`less than or equal to ${byteLength}`);
    });

    test("fromBytes throws for invalid RLP", () => {
        expect(() => {
            const buf = new Buffer(byteLength + 1);
            buf[0] = 0xc0 + byteLength;
            buf.fill(0xff, 1, byteLength + 1);
            expect(Uxxx.fromBytes(buf)).toEqual(Uxxx.MAX_VALUE);
        }).toThrow(`RLP`);
    });

    test("isEqualTo", () => {
        expect(new Uxxx(0).isEqualTo(new Uxxx(0))).toEqual(true);
        expect(new Uxxx(1000000).isEqualTo(new Uxxx(1000000))).toEqual(true);
        expect(
            new Uxxx("100000000000000000").isEqualTo(
                new Uxxx("100000000000000000")
            )
        ).toEqual(true);
    });

    test("rlpBytes", () => {
        expect(new Uxxx(0).rlpBytes()).toEqual(Buffer.from([0x80]));
        expect(new Uxxx(10).rlpBytes()).toEqual(Buffer.from([0x0a]));
        expect(new Uxxx(255).rlpBytes()).toEqual(Buffer.from([0x81, 0xff]));
        expect(new Uxxx(1000).rlpBytes()).toEqual(
            Buffer.from([0x82, 0x03, 0xe8])
        );
        expect(new Uxxx(100000).rlpBytes()).toEqual(
            Buffer.from([0x83, 0x01, 0x86, 0xa0])
        );
        expect(new Uxxx(10000000).rlpBytes()).toEqual(
            Buffer.from([0x83, 0x98, 0x96, 0x80])
        );
        expect(new Uxxx("1000000000").rlpBytes()).toEqual(
            Buffer.from([0x84, 0x3b, 0x9a, 0xca, 0x00])
        );
        expect(new Uxxx("1000000000000").rlpBytes()).toEqual(
            Buffer.from([0x85, 0xe8, 0xd4, 0xa5, 0x10, 0x00])
        );
    });

    test("toEncodeObject", () => {
        expect(new Uxxx(0).toEncodeObject()).toBe(0);
        expect(new Uxxx(0xf).toEncodeObject()).toBe("0x0f");
        expect(new Uxxx(0xff).toEncodeObject()).toBe("0xff");
        expect(new Uxxx(0xfff).toEncodeObject()).toBe("0x0fff");
    });

    test("toString", () => {
        expect(new Uxxx(0).toString()).toBe("0");
        expect(new Uxxx(0).toString(10)).toBe("0");
        expect(new Uxxx(0).toString(16)).toBe("0");
        expect(new Uxxx(0xff).toString(10)).toBe("255");
        expect(new Uxxx(0xff).toString(16)).toBe("ff");
    });

    test("toJSON", () => {
        expect(new Uxxx(0).toJSON()).toBe("0x0");
        expect(new Uxxx(0xff).toJSON()).toBe("0xff");
    });

    test("plus", () => {
        expect(Uxxx.plus(10, 5)).toEqual(new Uxxx(10 + 5));
        expect(() => {
            Uxxx.plus(Uxxx.MAX_VALUE, 1);
        }).toThrow("overflow");
        expect(() => {
            Uxxx.plus(-1, 0);
        }).toThrow(className);

        let a = new Uxxx(10);
        let b = new Uxxx(5);
        expect(a.plus(b)).toEqual(new Uxxx(15));
        a = new Uxxx(Uxxx.MAX_VALUE);
        b = new Uxxx(1);
        expect(() => {
            a.plus(b);
        }).toThrow("overflow");
    });

    test("minus", () => {
        expect(Uxxx.minus(10, 5)).toEqual(new Uxxx(10 - 5));
        expect(() => {
            Uxxx.minus(5, 10);
        }).toThrow("underflow");
        expect(() => {
            Uxxx.minus(-1, -1);
        }).toThrow(className);

        let a = new Uxxx(10);
        let b = new Uxxx(5);
        expect(a.minus(b)).toEqual(new Uxxx(10 - 5));
        a = new Uxxx(5);
        b = new Uxxx(10);
        expect(() => {
            a.minus(b);
        }).toThrow("underflow");
    });

    test("times", () => {
        expect(Uxxx.times(10, 5)).toEqual(new Uxxx(10 * 5));
        expect(Uxxx.times(Uxxx.MAX_VALUE, 0)).toEqual(new Uxxx(0));
        expect(Uxxx.times(Uxxx.MAX_VALUE, 1)).toEqual(Uxxx.MAX_VALUE);
        expect(() => {
            Uxxx.times(Uxxx.MAX_VALUE, 2);
        }).toThrow("overflow");
        expect(() => {
            Uxxx.times(-1, -1);
        }).toThrow(className);

        let a = new Uxxx(10);
        let b = new Uxxx(5);
        expect(a.times(b)).toEqual(new Uxxx(10 * 5));
        a = new Uxxx(Uxxx.MAX_VALUE);
        b = new Uxxx(0);
        expect(a.times(b)).toEqual(new Uxxx(0));
        a = new Uxxx(Uxxx.MAX_VALUE);
        b = new Uxxx(1);
        expect(a.times(b)).toEqual(Uxxx.MAX_VALUE);
        a = new Uxxx(Uxxx.MAX_VALUE);
        b = new Uxxx(2);
        expect(() => {
            a.times(b);
        }).toThrow("overflow");
    });

    test("idiv", () => {
        expect(Uxxx.idiv(10, 5)).toEqual(new Uxxx(10 / 5));
        expect(Uxxx.idiv(14, 5)).toEqual(new Uxxx(2));
        expect(() => {
            Uxxx.idiv(10, 0);
        }).toThrow("Divided by 0");
        expect(() => {
            Uxxx.idiv(-1, -1);
        }).toThrow(className);

        let a = new Uxxx(10);
        let b = new Uxxx(5);
        expect(a.idiv(b)).toEqual(new Uxxx(10 / 5));
        a = new Uxxx(14);
        b = new Uxxx(5);
        expect(a.idiv(b)).toEqual(new Uxxx(2));
        a = new Uxxx(10);
        b = new Uxxx(0);
        expect(() => {
            a.idiv(b);
        }).toThrow("Divided by 0");
    });

    test("mod", () => {
        expect(Uxxx.mod(10, 5)).toEqual(new Uxxx(0));
        expect(Uxxx.mod(14, 5)).toEqual(new Uxxx(4));
        expect(() => {
            Uxxx.mod(10, 0);
        }).toThrow("Divided by 0");
        expect(() => {
            Uxxx.mod(-1, -1);
        }).toThrow(className);

        let a = new Uxxx(10);
        let b = new Uxxx(5);
        expect(a.mod(b)).toEqual(new Uxxx(0));
        a = new Uxxx(14);
        b = new Uxxx(5);
        expect(a.mod(b)).toEqual(new Uxxx(4));
        a = new Uxxx(10);
        b = new Uxxx(0);
        expect(() => {
            a.mod(b);
        }).toThrow("Divided by 0");
    });

    test("Comparison", () => {
        expect(new Uxxx(11).gt(10)).toBe(true);
        expect(new Uxxx(10).gt(10)).toBe(false);
        expect(new Uxxx(9).gt(10)).toBe(false);
        expect(new Uxxx(11).isGreaterThan(10)).toBe(true);
        expect(new Uxxx(10).isGreaterThan(10)).toBe(false);
        expect(new Uxxx(9).isGreaterThan(10)).toBe(false);

        expect(new Uxxx(11).gte(10)).toBe(true);
        expect(new Uxxx(10).gte(10)).toBe(true);
        expect(new Uxxx(9).gte(10)).toBe(false);
        expect(new Uxxx(11).isGreaterThanOrEqualTo(10)).toBe(true);
        expect(new Uxxx(10).isGreaterThanOrEqualTo(10)).toBe(true);
        expect(new Uxxx(9).isGreaterThanOrEqualTo(10)).toBe(false);

        expect(new Uxxx(11).lt(10)).toBe(false);
        expect(new Uxxx(10).lt(10)).toBe(false);
        expect(new Uxxx(9).lt(10)).toBe(true);
        expect(new Uxxx(11).isLessThan(10)).toBe(false);
        expect(new Uxxx(10).isLessThan(10)).toBe(false);
        expect(new Uxxx(9).isLessThan(10)).toBe(true);

        expect(new Uxxx(11).lte(10)).toBe(false);
        expect(new Uxxx(10).lte(10)).toBe(true);
        expect(new Uxxx(9).lte(10)).toBe(true);
        expect(new Uxxx(11).isLessThanOrEqualTo(10)).toBe(false);
        expect(new Uxxx(10).isLessThanOrEqualTo(10)).toBe(true);
        expect(new Uxxx(9).isLessThanOrEqualTo(10)).toBe(true);
    });

    test("toLocaleString", () => {
        expect(new Uxxx(1234567).toLocaleString()).toBe("1,234,567");
        expect(new Uxxx(123).toLocaleString()).toBe("123");
    });
});
