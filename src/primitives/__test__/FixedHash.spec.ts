import { H160, H256, H512 } from "../index";

test("0x-prefix", () => {
    const value = new H160("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
    expect(H160.fromBytes(value.rlpBytes())).toEqual(value);
});

test("H160", () => {
    const value = new H160("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
    expect(H160.fromBytes(value.rlpBytes())).toEqual(value);
});

test("H256", () => {
    const value = new H256("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
    expect(H256.fromBytes(value.rlpBytes())).toEqual(value);
});

test("H512", () => {
    const value = new H512("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
    expect(H512.fromBytes(value.rlpBytes())).toEqual(value);
});

test("compare H256", () => {
    const value = new H256("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
    const expected = new H256("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
    expect(expected.isEqualTo(value)).toEqual(true);
});
