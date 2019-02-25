import { H256, PlatformAddress, U256, U64 } from "codechain-primitives";

import { Block } from "../Block";
import { Pay } from "../transaction/Pay";

test("toJSON", () => {
    const secret = new H256(
        "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd"
    );
    const pay = new Pay(
        PlatformAddress.fromAccountId(
            "0x2222222222222222222222222222222222222222",
            { networkId: "tc" }
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
        author: PlatformAddress.fromAccountId(
            "1111111111111111111111111111111111111111",
            { networkId: "tc" }
        ),
        extraData: [],
        transactionsRoot: new H256(
            "1111111111111111111111111111111111111111111111111111111111111111"
        ),
        stateRoot: new H256(
            "2222222222222222222222222222222222222222222222222222222222222222"
        ),
        resultsRoot: new H256(
            "3333333333333333333333333333333333333333333333333333333333333333"
        ),
        score: new U256(3),
        seal: [],
        hash: new H256(
            "4444444444444444444444444444444444444444444444444444444444444444"
        ),
        transactions: [p]
    });
    expect(Block.fromJSON(block.toJSON())).toEqual(block);
});
