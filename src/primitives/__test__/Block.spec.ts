import { Block, H256, H160, U256, Parcel } from "..";
import { PaymentTransaction } from "../transaction";

test("toJSON", () => {
    const t = new PaymentTransaction({
        nonce: new U256(22),
        address: new H160("0x2222222222222222222222222222222222222222"),
        value: new U256(11),
    });
    const p = new Parcel(new U256(33), new U256(44), 17, t);
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