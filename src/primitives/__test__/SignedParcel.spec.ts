import { U256, H160, Parcel, H256, SignedParcel } from "..";
import { privateKeyToAddress } from "../../utils";

test("toJSON", () => {
    const secret = new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd");
    const sender = new H160(privateKeyToAddress(secret.value));
    const p = Parcel.payment(new U256(33), new U256(44), 17, new H160("0x0000000000000000000000000000000000000000"), new U256(11)).sign(secret);
    expect(SignedParcel.fromJSON(p.toJSON())).toEqual(p);
});
