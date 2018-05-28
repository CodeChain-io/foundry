import { PaymentTransaction, U256, H160, Parcel } from "../..";

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
