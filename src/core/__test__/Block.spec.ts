import { PlatformAddress } from "codechain-primitives";

import { getAccountIdFromPrivate } from "../../utils";
import { Block } from "../Block";
import { H256 } from "../H256";
import { Parcel } from "../Parcel";
import { U256 } from "../U256";
import { U64 } from "../U64";

test("toJSON", () => {
    const secret = new H256(
        "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd"
    );
    const sender = PlatformAddress.fromAccountId(
        getAccountIdFromPrivate(secret.value),
        { networkId: "tc" }
    );
    const p = Parcel.pay(
        "tc",
        PlatformAddress.fromAccountId(
            "0x2222222222222222222222222222222222222222",
            { networkId: "tc" }
        ),
        new U64(11)
    ).sign({
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
        extraData: Buffer.from([]),
        parcelsRoot: new H256(
            "1111111111111111111111111111111111111111111111111111111111111111"
        ),
        stateRoot: new H256(
            "2222222222222222222222222222222222222222222222222222222222222222"
        ),
        invoicesRoot: new H256(
            "3333333333333333333333333333333333333333333333333333333333333333"
        ),
        score: new U256(3),
        seal: [],
        hash: new H256(
            "4444444444444444444444444444444444444444444444444444444444444444"
        ),
        parcels: [p]
    });
    expect(Block.fromJSON(block.toJSON())).toEqual(block);
});
