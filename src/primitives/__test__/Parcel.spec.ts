import { U256, H160, H256, Parcel } from "..";
import { PaymentTransaction } from "../transaction";

test("rlp", () => {
    const t = new Parcel(new U256(0), new U256(0), 1);
    expect(t.rlpBytes()).toEqual(Buffer.from([0xc4, 0x80, 0x80, 0xC0, 0x01]));
});

test("hash", () => {
    const t = new Parcel(new U256(0), new U256(0), 1);
    expect(t.hash()).toEqual(new H256("7093105182f0dcebd2e77dd4c9a6ba8ae69e858a61576ebea1586749d3f80b14"));
});

test("sign", () => {
    const t = new Parcel(new U256(0), new U256(0), 1);
    const signed = t.sign(new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd"));
    const { v, r, s } = signed.signature();
    expect(v).toBe(0);
    expect(r).toEqual(new U256("0x34bc75451413f2006a526e798c54e798da5311643159fbe4176cfe9524cd0249"));
    expect(s).toEqual(new U256("0x5307f57dc705e3ae9f6ad71c3a895a278cbb579b7a05d9b967f308e0c6467069"));
});

test("signed hash", () => {
    const t = new Parcel(new U256(0), new U256(0), 1);
    const signed = t.sign(new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd"));
    expect(signed.hash()).toEqual(new H256("28a2c15aca7055fb307aa56f9fa4b9087942f55f570f82cd50645ee453357409"));
});

test("toJSON", () => {
    const t = new PaymentTransaction({
        nonce: new U256(22),
        sender: new H160("0x0000000000000000000000000000000000000000"),
        receiver: new H160("0x0000000000000000000000000000000000000000"),
        value: new U256(11),
    });
    const p = new Parcel(new U256(33), new U256(44), 17, t);
    expect(Parcel.fromJSON(p.toJSON())).toEqual(p);
});
