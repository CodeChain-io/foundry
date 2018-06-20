import { U256, H160, H256, Parcel } from "..";

test("rlp", () => {
    const t = Parcel.transactions(new U256(0), new U256(0), 1);
    expect(t.rlpBytes()).toEqual(Buffer.from([0xc6, 0x80, 0x80, 0x01, 0xC2, 0x01, 0xC0]));
});

test("hash", () => {
    const t = Parcel.transactions(new U256(0), new U256(0), 1);
    expect(t.hash()).toEqual(new H256("793ab19e6663f78a1ac52e440347b8efe0d74318eccee020972a25adb926b3fa"));
});

test("sign", () => {
    const t = Parcel.transactions(new U256(0), new U256(0), 1);
    const signed = t.sign(new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd"));
    const { v, r, s } = signed.signature();
    expect(v).toBe(0);
    expect(r).toEqual(new U256("0x3824b636cd92324253b13bcb7c674ecb44e7dbb22cd438c39a5529cd2a923e0b"));
    expect(s).toEqual(new U256("0x1691d9f073d407f71e5f0ca6450b6f206a6ddb97a9ee22951ec1d75eff4d30e6"));
});

test("signed hash", () => {
    const t = Parcel.transactions(new U256(0), new U256(0), 1);
    const signed = t.sign(new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd"));
    expect(signed.hash()).toEqual(new H256("aac5d5de21a239e335b10c43d4b7666fdd2477963c72bc54b87cd4ab6a9d43d7"));
});

test("toJSON", () => {
    const p = Parcel.payment(new U256(33), new U256(44), 17, new H160("0x0000000000000000000000000000000000000000"), new U256(11));
    expect(Parcel.fromJSON(p.toJSON())).toEqual(p);
});
