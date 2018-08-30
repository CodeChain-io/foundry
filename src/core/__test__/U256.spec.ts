import { U256 } from "../U256";
const RLP = require("rlp");

test("rlpBytes", () => {
    expect(new U256(0).rlpBytes()).toEqual(Buffer.from([0x80]));
    expect(new U256(10).rlpBytes()).toEqual(Buffer.from([0x0a]));
    expect(new U256(255).rlpBytes()).toEqual(Buffer.from([0x81, 0xff]));
    expect(new U256(1000).rlpBytes()).toEqual(Buffer.from([0x82, 0x03, 0xe8]));
    expect(new U256(100000).rlpBytes()).toEqual(
        Buffer.from([0x83, 0x01, 0x86, 0xa0])
    );
    expect(new U256(10000000).rlpBytes()).toEqual(
        Buffer.from([0x83, 0x98, 0x96, 0x80])
    );
    expect(new U256("1000000000").rlpBytes()).toEqual(
        Buffer.from([0x84, 0x3b, 0x9a, 0xca, 0x00])
    );
    expect(new U256("1000000000000").rlpBytes()).toEqual(
        Buffer.from([0x85, 0xe8, 0xd4, 0xa5, 0x10, 0x00])
    );
});

test("fromBytes", () => {
    let a;
    a = new U256(0);
    expect(U256.fromBytes(a.rlpBytes())).toEqual(a);
    a = new U256(255);
    expect(U256.fromBytes(a.rlpBytes())).toEqual(a);
    a = new U256(1000);
    expect(U256.fromBytes(a.rlpBytes())).toEqual(a);
    a = new U256("1000000000000");
    expect(U256.fromBytes(a.rlpBytes())).toEqual(a);
});

test("increase", () => {
    const a = new U256(0);
    const b = a.increase();
    expect(a).toEqual(new U256(0));
    expect(b).toEqual(new U256(1));
});

test("isEqualTo", () => {
    expect(new U256(0).isEqualTo(new U256(0))).toEqual(true);
    expect(new U256(1000000).isEqualTo(new U256(1000000))).toEqual(true);
    expect(
        new U256("100000000000000000").isEqualTo(new U256("100000000000000000"))
    ).toEqual(true);
});

test("ensure", () => {
    expect(() => {
        U256.ensure(undefined);
    }).toThrow();
    expect(U256.ensure(10)).toEqual(new U256(10));
    expect(U256.ensure("10")).toEqual(new U256(10));
    expect(U256.ensure("0xA")).toEqual(new U256(10));
    expect(U256.ensure(new U256(10))).toEqual(new U256(10));
});
