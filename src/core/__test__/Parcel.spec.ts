import { H160 } from "../H160";
import { H256 } from "../H256";
import { U256 } from "../U256";
import { Parcel } from "../Parcel";

test("rlp", () => {
    const t = Parcel.transactions(new U256(0), new U256(0), 1);
    expect(t.rlpBytes()).toEqual(Buffer.from([248, 78, 128, 128, 1, 248, 73, 1, 192, 248, 69, 248, 67, 128, 160, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 160, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]));
});

test("hash", () => {
    const t = Parcel.transactions(new U256(0), new U256(0), 1);
    expect(t.hash()).toEqual(new H256("78850c22e15642a364d57c2a9e5df97bb2876ee32fdf93da1e11afcd2d586245"));
});

test("sign", () => {
    const t = Parcel.transactions(new U256(0), new U256(0), 1);
    const signed = t.sign(new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd"));
    const { v, r, s } = signed.signature();
    expect(v).toBe(1);
    expect(r).toEqual(new U256("0x4ec506266b9945c152b325d8155c6ee05b9602272a87c0f9bf6180495e0c0cc1"));
    expect(s).toEqual(new U256("0x4e1c05949e04cec49db5185f0f6dbfcc56ac83a1eae3fb6d45ae4b60d382ca3d"));
});

test("signed hash", () => {
    const t = Parcel.transactions(new U256(0), new U256(0), 1);
    const signed = t.sign(new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd"));
    expect(signed.hash()).toEqual(new H256("ec67fb2529da6f438d5bbf45c8025bcbcfa4d87c2d6ca2b36a25501e3cadc665"));
});

test("toJSON", () => {
    const p = Parcel.payment(new U256(33), new U256(44), 17, new H160("0x0000000000000000000000000000000000000000"), new U256(11));
    expect(Parcel.fromJSON(p.toJSON())).toEqual(p);
});
