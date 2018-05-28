import { Block, H256, H160, U256, Parcel } from "..";
import { PaymentTransaction } from "../transaction";
import { privateKeyToAddress } from "../../utils";

test("toJSON", () => {
    const secret = new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd");
    const sender = new H160(privateKeyToAddress(secret.value));
    const t = new PaymentTransaction({
        nonce: new U256(22),
        sender,
        receiver: new H160("0x2222222222222222222222222222222222222222"),
        value: new U256(11),
    });
    const p = new Parcel(new U256(33), new U256(44), 17, t).sign(secret);
    const block = new Block({
        parentHash: new H256("0000000000000000000000000000000000000000000000000000000000000000"),
        timestamp: 1,
        number: 2,
        author: new H160("1111111111111111111111111111111111111111"),
        extraData: Buffer.from([]),
        parcelsRoot: new H256("1111111111111111111111111111111111111111111111111111111111111111"),
        stateRoot: new H256("2222222222222222222222222222222222222222222222222222222222222222"),
        invoicesRoot: new H256("3333333333333333333333333333333333333333333333333333333333333333"),
        score: new U256(3),
        seal: [],
        hash: new H256("4444444444444444444444444444444444444444444444444444444444444444"),
        parcels: [p],
    });
    expect(Block.fromJSON(block.toJSON())).toEqual(block);
});
