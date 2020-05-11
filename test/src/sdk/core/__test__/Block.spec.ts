import { expect } from "chai";
import "mocha";
import { Address, H256, H512, U64 } from "../../../primitives/src";
import { Block } from "../Block";
import { Pay } from "../transaction/Pay";

it("toJSON", () => {
    const secret = new H512(
        "9af28f6fd6a1170dbee2cb8c34abab0408e6d811d212cdcde23f72473eb0d97ad7a6d266837c1c591383b90d835068b9ed58dd3bcebd6e285911f58e40ce413c"
    );
    const pay = new Pay(
        Address.fromPublic(
            "0x2222222222222222222222222222222222222222222222222222222222222222",
            {
                networkId: "tc"
            }
        ),
        new U64(11),
        "tc"
    );

    const p = pay.sign({
        secret,
        fee: 33,
        seq: 44
    });
    const block = new Block({
        parentHash: new H256(
            "0000000000000000000000000000000000000000000000000000000000000000"
        ),
        timestamp: 1,
        number: 2,
        author: Address.fromPublic(
            "1111111111111111111111111111111111111111111111111111111111111111",
            { networkId: "tc" }
        ),
        lastCommittedValidators: [],
        extraData: [],
        transactionsRoot: new H256(
            "1111111111111111111111111111111111111111111111111111111111111111"
        ),
        stateRoot: new H256(
            "2222222222222222222222222222222222222222222222222222222222222222"
        ),
        nextValidatorSetHash: new H256(
            "3333333333333333333333333333333333333333333333333333333333333333"
        ),
        seal: [],
        hash: new H256(
            "4444444444444444444444444444444444444444444444444444444444444444"
        ),
        transactions: [p]
    });
    expect(Block.fromJSON(block.toJSON())).deep.equal(block);
});
