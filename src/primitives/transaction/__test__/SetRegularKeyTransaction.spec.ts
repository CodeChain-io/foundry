import { SetRegularKeyTransaction } from "..";
import { H160, H512, U256 } from "../..";

test("SetRegularKeyTransaction toJSON", () => {
    const t = new SetRegularKeyTransaction({
        address: new H160("0x0000000000000000000000000000000000000000"),
        nonce: new U256(22),
        key: new H512("22222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222"),
    });
    expect(SetRegularKeyTransaction.fromJSON(t.toJSON())).toEqual(t);
});
