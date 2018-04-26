import H160 from "../H160";
import H256 from "../H256";
import H512 from "../H512";

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
