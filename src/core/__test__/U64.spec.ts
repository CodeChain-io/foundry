import { U64 } from "../U64";

test("rlpBytes", () => {
    expect(new U64(0).rlpBytes()).toEqual(Buffer.from([0x80]));
    expect(new U64(10).rlpBytes()).toEqual(Buffer.from([0x0a]));
    expect(new U64(255).rlpBytes()).toEqual(Buffer.from([0x81, 0xff]));
    expect(new U64(1000).rlpBytes()).toEqual(Buffer.from([0x82, 0x03, 0xe8]));
    expect(new U64(100000).rlpBytes()).toEqual(
        Buffer.from([0x83, 0x01, 0x86, 0xa0])
    );
    expect(new U64(10000000).rlpBytes()).toEqual(
        Buffer.from([0x83, 0x98, 0x96, 0x80])
    );
    expect(new U64("1000000000").rlpBytes()).toEqual(
        Buffer.from([0x84, 0x3b, 0x9a, 0xca, 0x00])
    );
    expect(new U64("1000000000000").rlpBytes()).toEqual(
        Buffer.from([0x85, 0xe8, 0xd4, 0xa5, 0x10, 0x00])
    );
});

test("fromBytes", () => {
    let a;
    a = new U64(0);
    expect(U64.fromBytes(a.rlpBytes())).toEqual(a);
    a = new U64(255);
    expect(U64.fromBytes(a.rlpBytes())).toEqual(a);
    a = new U64(1000);
    expect(U64.fromBytes(a.rlpBytes())).toEqual(a);
    a = new U64("1000000000000");
    expect(U64.fromBytes(a.rlpBytes())).toEqual(a);
});

test("plus", () => {
    const a = new U64(0);
    const b = U64.plus(a, 1);
    expect(a).toEqual(new U64(0));
    expect(b).toEqual(new U64(1));
});

test("isEqualTo", () => {
    expect(new U64(0).isEqualTo(new U64(0))).toEqual(true);
    expect(new U64(1000000).isEqualTo(new U64(1000000))).toEqual(true);
    expect(
        new U64("100000000000000000").isEqualTo(new U64("100000000000000000"))
    ).toEqual(true);
});

test("ensure", () => {
    expect(() => {
        U64.ensure(undefined);
    }).toThrow();
    expect(U64.ensure(10)).toEqual(new U64(10));
    expect(U64.ensure("10")).toEqual(new U64(10));
    expect(U64.ensure("0xA")).toEqual(new U64(10));
    expect(U64.ensure(new U64(10))).toEqual(new U64(10));
});
