import BigNumber from "bignumber.js";
import "mocha";
import { expect } from "chai";
import { U128, U256, U64 } from "../src";

const TOO_LARGE =
    "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF";
([
    [U64, "U64", 8],
    [U128, "U128", 16],
    [U256, "U256", 32]
] as [any, string, number][]).forEach(args => {
    const [Uxxx, className, byteLength] = args;
    describe(`${className} ${byteLength}`, () => {
        it("import", () => {
            expect(typeof Uxxx).equal("function");
        });

        it("require", () => {
            const obj = require("../src");
            expect(typeof obj[className]).equal("function");
        });

        it("new", () => {
            expect(new Uxxx(16).eq(new Uxxx("16"))).true;
            expect(new Uxxx(16).eq(new Uxxx("0x10"))).true;
            expect(new Uxxx(16).eq(new Uxxx(new BigNumber(16)))).true;
            expect(new Uxxx(16).eq(new Uxxx(new BigNumber("16")))).true;
            expect(new Uxxx(16).eq(new Uxxx(new BigNumber("0x10")))).true;

            expect(() => new Uxxx(TOO_LARGE)).throw();

            if (Uxxx === U256) {
                expect(new Uxxx(16).eq(new U256(new U64(16)))).true;
                expect(new Uxxx(16).eq(new U256(new U128(16)))).true;
            }
            if (Uxxx === U128) {
                expect(new Uxxx(16).eq(new U128(new U64(16)))).true;
            }
        });

        it("check", () => {
            expect(Uxxx.check(undefined as any)).false;
            expect(Uxxx.check(null as any)).false;
            expect(Uxxx.check(-1)).false;
            expect(Uxxx.check(0.5)).false;
            expect(Uxxx.check(0.5)).false;

            expect(Uxxx.check(new BigNumber(-1))).false;
            expect(Uxxx.check(new BigNumber(0.5))).false;
            expect(Uxxx.check(new BigNumber(TOO_LARGE))).false;

            expect(Uxxx.check(0)).true;
            expect(Uxxx.check("0")).true;
            expect(Uxxx.check("0x0")).true;

            if (byteLength >= 32) {
                expect(Uxxx.check(new U256(0))).true;
            }
            if (byteLength >= 16) {
                expect(Uxxx.check(new U128(0))).true;
            }
            if (byteLength >= 8) {
                expect(Uxxx.check(new U64(0))).true;
            }
        });

        it("ensure", () => {
            expect(() => {
                Uxxx.ensure(undefined as any);
            }).throw();
            expect(Uxxx.ensure(10)).deep.equal(new Uxxx(10));
            expect(Uxxx.ensure("10")).deep.equal(new Uxxx(10));
            expect(Uxxx.ensure("0xA")).deep.equal(new Uxxx(10));
            expect(Uxxx.ensure(new Uxxx(10))).deep.equal(new Uxxx(10));

            if (Uxxx === U256) {
                expect(Uxxx.ensure(new U64(10))).deep.equal(new Uxxx(10));
            }
        });

        it("fromBytes", () => {
            let a;
            a = new Uxxx(0);
            expect(Uxxx.fromBytes(a.rlpBytes())).deep.equal(a);
            a = new Uxxx(1);
            expect(Uxxx.fromBytes(a.rlpBytes())).deep.equal(a);
            a = new Uxxx(0x79);
            expect(Uxxx.fromBytes(a.rlpBytes())).deep.equal(a);
            a = new Uxxx(255);
            expect(Uxxx.fromBytes(a.rlpBytes())).deep.equal(a);
            a = new Uxxx(1000);
            expect(Uxxx.fromBytes(a.rlpBytes())).deep.equal(a);
            a = new Uxxx("1000000000000");
            expect(Uxxx.fromBytes(a.rlpBytes())).deep.equal(a);
            const buf = new Buffer(byteLength + 1);
            buf[0] = 0x80 + byteLength;
            buf.fill(0xff, 1, byteLength + 1);
            expect(Uxxx.fromBytes(buf)).deep.equal(Uxxx.MAX_VALUE);
        });

        it("fromBytes throws for oversize Buffer", () => {
            expect(() => {
                const buf = new Buffer(byteLength + 2);
                buf[0] = 0x80 + byteLength + 1;
                buf.fill(0xff, 1, byteLength + 2);
                expect(Uxxx.fromBytes(buf)).deep.equal(Uxxx.MAX_VALUE);
            }).throw(`less than or equal to ${byteLength}`);
        });

        it("fromBytes throws for invalid RLP", () => {
            expect(() => {
                const buf = new Buffer(byteLength + 1);
                buf[0] = 0xc0 + byteLength;
                buf.fill(0xff, 1, byteLength + 1);
                expect(Uxxx.fromBytes(buf)).equal(Uxxx.MAX_VALUE);
            }).throw(`RLP`);
        });

        it("isEqualTo", () => {
            expect(new Uxxx(0).isEqualTo(new Uxxx(0))).true;
            expect(new Uxxx(1000000).isEqualTo(new Uxxx(1000000))).true;
            expect(
                new Uxxx("100000000000000000").isEqualTo(
                    new Uxxx("100000000000000000")
                )
            ).equal(true);
        });

        it("rlpBytes", () => {
            expect(new Uxxx(0).rlpBytes()).deep.equal(Buffer.from([0x80]));
            expect(new Uxxx(10).rlpBytes()).deep.equal(Buffer.from([0x0a]));
            expect(new Uxxx(255).rlpBytes()).deep.equal(
                Buffer.from([0x81, 0xff])
            );
            expect(new Uxxx(1000).rlpBytes()).deep.equal(
                Buffer.from([0x82, 0x03, 0xe8])
            );
            expect(new Uxxx(100000).rlpBytes()).deep.equal(
                Buffer.from([0x83, 0x01, 0x86, 0xa0])
            );
            expect(new Uxxx(10000000).rlpBytes()).deep.equal(
                Buffer.from([0x83, 0x98, 0x96, 0x80])
            );
            expect(new Uxxx("1000000000").rlpBytes()).deep.equal(
                Buffer.from([0x84, 0x3b, 0x9a, 0xca, 0x00])
            );
            expect(new Uxxx("1000000000000").rlpBytes()).deep.equal(
                Buffer.from([0x85, 0xe8, 0xd4, 0xa5, 0x10, 0x00])
            );
        });

        it("toEncodeObject", () => {
            expect(new Uxxx(0).toEncodeObject()).equal(0);
            expect(new Uxxx(0xf).toEncodeObject()).equal("0x0f");
            expect(new Uxxx(0xff).toEncodeObject()).equal("0xff");
            expect(new Uxxx(0xfff).toEncodeObject()).equal("0x0fff");
        });

        it("toString", () => {
            expect(new Uxxx(0).toString()).equal("0");
            expect(new Uxxx(0).toString(10)).equal("0");
            expect(new Uxxx(0).toString(16)).equal("0");
            expect(new Uxxx(0xff).toString(10)).equal("255");
            expect(new Uxxx(0xff).toString(16)).equal("ff");
        });

        it("toJSON", () => {
            expect(new Uxxx(0).toJSON()).equal("0x0");
            expect(new Uxxx(0xff).toJSON()).equal("0xff");
        });

        it("plus", () => {
            expect(Uxxx.plus(10, 5).toString()).equal(
                new Uxxx(10 + 5).toString()
            );
            expect(() => {
                Uxxx.plus(Uxxx.MAX_VALUE, 1);
            }).throw("overflow");
            expect(() => {
                Uxxx.plus(-1, 0);
            }).throw(className);

            let a = new Uxxx(10);
            let b = new Uxxx(5);
            expect(a.plus(b).toString()).equal(new Uxxx(15).toString());
            a = new Uxxx(Uxxx.MAX_VALUE);
            b = new Uxxx(1);
            expect(() => {
                a.plus(b);
            }).throw("overflow");
        });

        it("minus", () => {
            expect(Uxxx.minus(10, 5).toString()).equal(
                new Uxxx(10 - 5).toString()
            );
            expect(() => {
                Uxxx.minus(5, 10);
            }).throw("underflow");
            expect(() => {
                Uxxx.minus(-1, -1);
            }).throw(className);

            let a = new Uxxx(10);
            let b = new Uxxx(5);
            expect(a.minus(b).toString()).equal(new Uxxx(10 - 5).toString());
            a = new Uxxx(5);
            b = new Uxxx(10);
            expect(() => {
                a.minus(b);
            }).throw("underflow");
        });

        it("times", () => {
            expect(Uxxx.times(10, 5).toString()).equal(
                new Uxxx(10 * 5).toString()
            );
            expect(Uxxx.times(Uxxx.MAX_VALUE, 0).toString()).equal(
                new Uxxx(0).toString()
            );
            expect(Uxxx.times(Uxxx.MAX_VALUE, 1).toString()).equal(
                Uxxx.MAX_VALUE.toString()
            );
            expect(() => {
                Uxxx.times(Uxxx.MAX_VALUE, 2);
            }).throw("overflow");
            expect(() => {
                Uxxx.times(-1, -1);
            }).throw(className);

            let a = new Uxxx(10);
            let b = new Uxxx(5);
            expect(a.times(b).toString()).equal(new Uxxx(10 * 5).toString());
            a = new Uxxx(Uxxx.MAX_VALUE);
            b = new Uxxx(0);
            expect(a.times(b).toString()).equal(new Uxxx(0).toString());
            a = new Uxxx(Uxxx.MAX_VALUE);
            b = new Uxxx(1);
            expect(a.times(b).toString()).equal(Uxxx.MAX_VALUE.toString());
            a = new Uxxx(Uxxx.MAX_VALUE);
            b = new Uxxx(2);
            expect(() => {
                a.times(b);
            }).throw("overflow");
        });

        it("idiv", () => {
            expect(Uxxx.idiv(10, 5).toString()).equal(
                new Uxxx(10 / 5).toString()
            );
            expect(Uxxx.idiv(14, 5).toString()).equal(new Uxxx(2).toString());
            expect(() => {
                Uxxx.idiv(10, 0);
            }).throw("Divided by 0");
            expect(() => {
                Uxxx.idiv(-1, -1);
            }).throw(className);

            let a = new Uxxx(10);
            let b = new Uxxx(5);
            expect(a.idiv(b).toString()).equal(new Uxxx(10 / 5).toString());
            a = new Uxxx(14);
            b = new Uxxx(5);
            expect(a.idiv(b).toString()).equal(new Uxxx(2).toString());
            a = new Uxxx(10);
            b = new Uxxx(0);
            expect(() => {
                a.idiv(b);
            }).throw("Divided by 0");
        });

        it("mod", () => {
            expect(Uxxx.mod(10, 5).toString()).equal(new Uxxx(0).toString());
            expect(Uxxx.mod(14, 5).toString()).equal(new Uxxx(4).toString());
            expect(() => {
                Uxxx.mod(10, 0);
            }).throw("Divided by 0");
            expect(() => {
                Uxxx.mod(-1, -1);
            }).throw(className);

            let a = new Uxxx(10);
            let b = new Uxxx(5);
            expect(a.mod(b).toString()).equal(new Uxxx(0).toString());
            a = new Uxxx(14);
            b = new Uxxx(5);
            expect(a.mod(b).toString()).equal(new Uxxx(4).toString());
            a = new Uxxx(10);
            b = new Uxxx(0);
            expect(() => {
                a.mod(b);
            }).throw("Divided by 0");
        });

        it("Comparison", () => {
            expect(new Uxxx(11).gt(10)).true;
            expect(new Uxxx(10).gt(10)).false;
            expect(new Uxxx(9).gt(10)).false;
            expect(new Uxxx(11).isGreaterThan(10)).true;
            expect(new Uxxx(10).isGreaterThan(10)).false;
            expect(new Uxxx(9).isGreaterThan(10)).false;

            expect(new Uxxx(11).gte(10)).true;
            expect(new Uxxx(10).gte(10)).true;
            expect(new Uxxx(9).gte(10)).false;
            expect(new Uxxx(11).isGreaterThanOrEqualTo(10)).true;
            expect(new Uxxx(10).isGreaterThanOrEqualTo(10)).true;
            expect(new Uxxx(9).isGreaterThanOrEqualTo(10)).false;

            expect(new Uxxx(11).lt(10)).false;
            expect(new Uxxx(10).lt(10)).false;
            expect(new Uxxx(9).lt(10)).true;
            expect(new Uxxx(11).isLessThan(10)).false;
            expect(new Uxxx(10).isLessThan(10)).false;
            expect(new Uxxx(9).isLessThan(10)).true;

            expect(new Uxxx(11).lte(10)).false;
            expect(new Uxxx(10).lte(10)).true;
            expect(new Uxxx(9).lte(10)).true;
            expect(new Uxxx(11).isLessThanOrEqualTo(10)).false;
            expect(new Uxxx(10).isLessThanOrEqualTo(10)).true;
            expect(new Uxxx(9).isLessThanOrEqualTo(10)).true;
        });

        it("toLocaleString", () => {
            expect(new Uxxx(1234567).toLocaleString()).equal("1,234,567");
            expect(new Uxxx(123).toLocaleString()).equal("123");
        });
    });
});
