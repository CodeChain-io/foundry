import { NoopTransaction } from "../transaction/index";
import { H256, U256, Parcel } from "../";

test("rlp", () => {
    const t = new Parcel(new U256(0), new U256(0), 1, new NoopTransaction());
    expect(t.rlpBytes()).toEqual(Buffer.from([0xc5, 0x80, 0x80, 0xC1, 0x80, 0x01]));
});

test("hash", () => {
    const t = new Parcel(new U256(0), new U256(0), 1, new NoopTransaction());
    expect(t.hash()).toEqual(new H256("0fe0daf8dc502e5d0217a9a98662b59397740e2af8251262980cffb895f985ae"));
});

test("sign", () => {
    const t = new Parcel(new U256(0), new U256(0), 1, new NoopTransaction());
    const signed = t.sign(new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd"));
    const { v, r, s } = signed.signature();
    expect(v).toBe(1 + 26);
    expect(r).toEqual(new U256("0x96884e18f9989a143696f81dfc8ded0ddaee11ff2fdb5424e22c07606580dc3b"));
    expect(s).toEqual(new U256("0x3479e7fc71a74d3a343c11c2a56a3c32f43863ea7f3b840fbbef29bf68ebd55f"));
});

test("signed hash", () => {
    const t = new Parcel(new U256(0), new U256(0), 1, new NoopTransaction());
    const signed = t.sign(new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd"));
    expect(signed.hash()).toEqual(new H256("274559e29521f50e79059a0c7f43a0f44a66c21251744d4b121fd7d74b5daca1"));
});
