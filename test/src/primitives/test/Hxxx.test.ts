import * as _ from "lodash";
import { H128, H160, H256, H512 } from "../src";
import "mocha";
import { expect } from "chai";

([
    [H128, "H128", 16],
    [H160, "H160", 20],
    [H256, "H256", 32],
    [H512, "H512", 64]
] as [any, string, number][]).forEach(args => {
    const [Hxxx, className, byteLength] = args;
    describe(`${className}, ${byteLength}`, () => {
        it("import", () => {
            expect(typeof Hxxx).equal("function");
        });

        it("require", () => {
            const obj = require("../src");
            expect(typeof obj[className]).equal("function");
        });

        it("new", () => {
            const zero = _.repeat("00", byteLength);
            expect(() => new Hxxx(zero)).not.throw();
            expect(() => new Hxxx(`0x${zero}`)).not.throw();
            expect(() => new Hxxx(zero + "0")).throw(String(byteLength));
            expect(() => new Hxxx(zero.substr(1))).throw(String(byteLength));
        });

        it("zero", () => {
            const zero = _.repeat("00", byteLength);
            expect(Hxxx.zero()).deep.equal(new Hxxx(zero));
        });

        it("check", () => {
            const zero = _.repeat("00", byteLength);
            expect(Hxxx.check(new Hxxx(zero))).true;
            expect(Hxxx.check(zero)).true;
            expect(Hxxx.check(zero.substr(1) + "F")).true;
            expect(Hxxx.check(zero.substr(1) + "f")).true;
            expect(Hxxx.check(zero.substr(1) + "g")).false;
            expect(Hxxx.check(zero.substr(1) + "g")).false;
            expect(Hxxx.check(zero + "0")).false;
        });

        it("ensure", () => {
            const zero = _.repeat("00", byteLength);
            expect(Hxxx.ensure(zero)).deep.equal(new Hxxx(zero));
            expect(Hxxx.ensure(new Hxxx(zero))).deep.equal(new Hxxx(zero));
        });

        describe("fromBytes", () => {
            it("zero", () => {
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

                expect(Hxxx.fromBytes(zeroBytes)).deep.equal(new Hxxx(zero));
            });

            it("FF", () => {
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

                expect(Hxxx.fromBytes(bytes)).deep.equal(new Hxxx(value));
            });
        });

        it("fromBytes throws", () => {
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
            }).throw("RLP");
        });

        it("isEqualTo", () => {
            const zero = _.repeat("00", byteLength);
            const one = _.repeat("00", byteLength - 1) + "01";

            expect(new Hxxx(zero).isEqualTo(new Hxxx(zero))).true;
            expect(new Hxxx(zero).isEqualTo(new Hxxx(one))).false;
        });

        it("rlpBytes", () => {
            const zero = _.repeat("00", byteLength);
            if (byteLength <= 55) {
                expect(new Hxxx(zero).rlpBytes()).deep.equal(
                    Buffer.from([
                        0x80 + byteLength,
                        ..._.times(byteLength, () => 0)
                    ])
                );
            } else if (byteLength <= 0xff) {
                expect(new Hxxx(zero).rlpBytes()).deep.equal(
                    Buffer.from([
                        0xb8,
                        byteLength,
                        ..._.times(byteLength, () => 0)
                    ])
                );
            } else {
                throw Error("Not implemented");
            }
        });

        it("toEncodeObject", () => {
            const zero = _.repeat("00", byteLength);
            expect(new Hxxx(zero).toEncodeObject()).equal(`0x${zero}`);
        });

        it("toString", () => {
            const zero = _.repeat("00", byteLength);
            expect(new Hxxx(zero).toString()).equal(zero);
        });

        it("toJSON", () => {
            const zero = _.repeat("00", byteLength);
            expect(new Hxxx(zero).toJSON()).equal(`0x${zero}`);
        });
    });
});
