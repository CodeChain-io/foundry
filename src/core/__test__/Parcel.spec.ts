import { H160 } from "../H160";
import { H256 } from "../H256";
import { U256 } from "../U256";
import { Parcel } from "../Parcel";

test("rlp", () => {
    const t = Parcel.transactions(1);
    t.setFee(0);
    t.setNonce(0);
    expect(t.rlpBytes()).toEqual(Buffer.from([248, 79, 128, 128, 1, 248, 74, 1, 192, 248, 69, 248, 67, 128, 160, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 160, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192]));
});

test("hash", () => {
    const t = Parcel.transactions(1);
    t.setFee(0);
    t.setNonce(0);
    expect(t.hash()).toEqual(new H256("9380648466574175d5363ff9411cf723a899ad8c75d975730fcc3169bea84f79"));
});

test("sign", () => {
    const t = Parcel.transactions(1);
    const signed = t.sign({
        secret: "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd",
        nonce: 0,
        fee: 0
    });
    const { v, r, s } = signed.signature();
    expect(v).toBe(1);
    expect(r.toEncodeObject()).toEqual(new U256("0xedfcde8b129c6d8faaaaef6a0dae4fd510a25f34dfb4e8abda326d895955611e").toEncodeObject());
    expect(s.toEncodeObject()).toEqual(new U256("0x65f81e9d365b4f79e35e4e4e86e93a383b4395b84220ae6a0c95960fd8e8f17c").toEncodeObject());
});

test("signed hash", () => {
    const t = Parcel.transactions(1);
    const signed = t.sign({
        secret: "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd",
        nonce: 0,
        fee: 0
    });
    expect(signed.hash()).toEqual(new H256("c69919c07a984792e0d5ecba84011cb74004ecc8fccc98f65f1f424a208cec32"));
});

test("toJSON", () => {
    const p = Parcel.payment(17, new H160("0x0000000000000000000000000000000000000000"), new U256(11));
    p.setFee(33);
    p.setNonce(44);
    expect(Parcel.fromJSON(p.toJSON())).toEqual(p);
});
