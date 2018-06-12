import { PaymentTransaction } from "..";
import { H160, U256 } from "../../";

test("PaymentTransaction toJSON", () => {
    const t = new PaymentTransaction({
        nonce: new U256(22),
        sender: new H160("0x0000000000000000000000000000000000000000"),
        receiver: new H160("0x0000000000000000000000000000000000000000"),
        value: new U256(11),
    });
    expect(PaymentTransaction.fromJSON(t.toJSON())).toEqual(t);
});
