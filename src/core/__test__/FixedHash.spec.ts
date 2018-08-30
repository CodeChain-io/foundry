import { H160 } from "../H160";
import { H256 } from "../H256";
import { H512 } from "../H512";

test("0x-prefix", () => {
    const value = new H160("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
    expect(H160.fromBytes(value.rlpBytes())).toEqual(value);
});

test("H160", () => {
    const value = new H160("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
    expect(H160.fromBytes(value.rlpBytes())).toEqual(value);
});

test("H256", () => {
    const value = new H256(
        "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
    );
    expect(H256.fromBytes(value.rlpBytes())).toEqual(value);
});

test("H512", () => {
    const value = new H512(
        "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
    );
    expect(H512.fromBytes(value.rlpBytes())).toEqual(value);
});

test("compare H256", () => {
    const value = new H256(
        "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
    );
    const expected = new H256(
        "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
    );
    expect(expected.isEqualTo(value)).toEqual(true);
});

test("ensure H160", () => {
    const a = H160.ensure("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
    const b = H160.ensure("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
    const c = H160.ensure(
        new H160("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF")
    );
    expect(a).toEqual(b);
    expect(b).toEqual(c);
});

test("ensure H256", () => {
    const a = H256.ensure(
        "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
    );
    const b = H256.ensure(
        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
    );
    const c = H256.ensure(
        new H256(
            "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
        )
    );
    expect(a).toEqual(b);
    expect(b).toEqual(c);
});

test("ensure H512", () => {
    const a = H512.ensure(
        "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
    );
    const b = H512.ensure(
        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
    );
    const c = H512.ensure(
        new H512(
            "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
        )
    );
    expect(a).toEqual(b);
    expect(b).toEqual(c);
});
