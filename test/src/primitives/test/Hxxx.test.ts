import * as _ from "lodash";

import { H128, H160, H256, H512 } from "..";

describe.each([
    [H128, "H128", 16],
    [H160, "H160", 20],
    [H256, "H256", 32],
    [H512, "H512", 64]
])("%p", (Hxxx, className, byteLength) => {
    test("import", () => {
        expect(typeof Hxxx).toBe("function");
    });

    test("require", () => {
        const obj = require("..");
        expect(typeof obj[className]).toBe("function");
    });

    test("new", () => {
        const zero = _.repeat("00", byteLength);
        expect(() => new Hxxx(zero)).not.toThrow();
        expect(() => new Hxxx(`0x${zero}`)).not.toThrow();
        expect(() => new Hxxx(zero + "0")).toThrow(String(byteLength));
        expect(() => new Hxxx(zero.substr(1))).toThrow(String(byteLength));
    });

    test("zero", () => {
        const zero = _.repeat("00", byteLength);
        expect(Hxxx.zero()).toEqual(new Hxxx(zero));
    });

    test("check", () => {
        const zero = _.repeat("00", byteLength);
        expect(Hxxx.check(new Hxxx(zero))).toBe(true);
        expect(Hxxx.check(zero)).toBe(true);
        expect(Hxxx.check(zero.substr(1) + "F")).toBe(true);
        expect(Hxxx.check(zero.substr(1) + "f")).toBe(true);
        expect(Hxxx.check(zero.substr(1) + "g")).toBe(false);
        expect(Hxxx.check(zero.substr(1) + "g")).toBe(false);
        expect(Hxxx.check(zero + "0")).toBe(false);
    });

    test("ensure", () => {
        const zero = _.repeat("00", byteLength);
        expect(Hxxx.ensure(zero)).toEqual(new Hxxx(zero));
        expect(Hxxx.ensure(new Hxxx(zero))).toEqual(new Hxxx(zero));
    });

    describe("fromBytes", () => {
        test("zero", () => {
            const zero = _.repeat("00", byteLength);
            let zeroBytes: Buffer;
            if (byteLength <= 55) {
                zeroBytes = Buffer.from([
                    0x80 + byteLength,
                    ..._.times(byteLength, () => 0)
                ]);
            } else if (byteLength <= 0xff) {
                zeroBytes = Buffer.from([
                    0xb8,
                    byteLength,
                    ..._.times(byteLength, () => 0)
                ]);
            } else {
                throw Error(`Not implemented`);
            }

            expect(Hxxx.fromBytes(zeroBytes)).toEqual(new Hxxx(zero));
        });

        test("FF", () => {
            const value = _.repeat("FF", byteLength);
            let bytes: Buffer;
            if (byteLength <= 55) {
                bytes = Buffer.from([
                    0x80 + byteLength,
                    ..._.times(byteLength, () => 255)
                ]);
            } else if (byteLength <= 0xff) {
                bytes = Buffer.from([
                    0xb8,
                    byteLength,
                    ..._.times(byteLength, () => 255)
                ]);
            } else {
                throw Error(`Not implemented`);
            }

            expect(Hxxx.fromBytes(bytes)).toEqual(new Hxxx(value));
        });
    });

    test("fromBytes throws", () => {
        let longerZeroBytes: Buffer;
        if (byteLength <= 55) {
            longerZeroBytes = Buffer.from([
                0x80 + byteLength + 1,
                ..._.times(byteLength + 1, () => 0)
            ]);
        } else if (byteLength <= 0xff) {
            longerZeroBytes = Buffer.from([
                0xb8,
                byteLength + 1,
                ..._.times(byteLength + 1, () => 0)
            ]);
        } else {
            throw Error(`Not implemented`);
        }

        expect(() => {
            Hxxx.fromBytes(longerZeroBytes);
        }).toThrow("RLP");
    });

    test("isEqualTo", () => {
        const zero = _.repeat("00", byteLength);
        const one = _.repeat("00", byteLength - 1) + "01";

        expect(new Hxxx(zero).isEqualTo(new Hxxx(zero))).toBe(true);
        expect(new Hxxx(zero).isEqualTo(new Hxxx(one))).toBe(false);
    });

    test("rlpBytes", () => {
        const zero = _.repeat("00", byteLength);
        if (byteLength <= 55) {
            expect(new Hxxx(zero).rlpBytes()).toEqual(
                Buffer.from([
                    0x80 + byteLength,
                    ..._.times(byteLength, () => 0)
                ])
            );
        } else if (byteLength <= 0xff) {
            expect(new Hxxx(zero).rlpBytes()).toEqual(
                Buffer.from([0xb8, byteLength, ..._.times(byteLength, () => 0)])
            );
        } else {
            throw Error("Not implemented");
        }
    });

    test("toEncodeObject", () => {
        const zero = _.repeat("00", byteLength);
        expect(new Hxxx(zero).toEncodeObject()).toEqual(`0x${zero}`);
    });

    test("toString", () => {
        const zero = _.repeat("00", byteLength);
        expect(new Hxxx(zero).toString()).toBe(zero);
    });

    test("toJSON", () => {
        const zero = _.repeat("00", byteLength);
        expect(new Hxxx(zero).toJSON()).toBe(`0x${zero}`);
    });
});
