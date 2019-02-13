import { H256, PlatformAddress, U256, U64 } from "codechain-primitives";
import { Pay } from "../transaction/Pay";

import { fromJSONToTransaction } from "../transaction/json";

test("rlp", () => {
    const t = new Pay(
        PlatformAddress.fromAccountId(
            "0x0000000000000000000000000000000000000000",
            { networkId: "tc" }
        ),
        new U64(11),
        "tc"
    );
    t.setFee(0);
    t.setSeq(0);
    expect(t.rlpBytes()).toEqual(
        Buffer.from([
            221,
            128,
            128,
            130,
            116,
            99,
            215,
            2,
            148,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            11
        ])
    );
});

test("hash", () => {
    const t = new Pay(
        PlatformAddress.fromAccountId(
            "0x0000000000000000000000000000000000000000",
            { networkId: "tc" }
        ),
        new U64(11),
        "tc"
    );
    t.setFee(0);
    t.setSeq(0);
    expect(t.hash()).toEqual(
        new H256(
            "3b578bebb32cae770ab1094d572a4721b624fc101bb88fbc580eeb2931f65665"
        )
    );
});

test("sign", () => {
    const pay = new Pay(
        PlatformAddress.fromAccountId(
            "0x0000000000000000000000000000000000000000",
            { networkId: "tc" }
        ),
        new U64(11),
        "tc"
    );
    const signed = pay.sign({
        secret:
            "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd",
        seq: 0,
        fee: 0
    });
    expect(signed.signature()).toBe(
        "3f9bcff484bd5f1d5549f912f9eeaf8c2fe349b257bde2b61fb1036013d4e44c204a4215d26cb879eaad2028fe1a7898e4cf9a5d979eb383e0a384140d6e04c101"
    );
});

test("signed hash", () => {
    const pay = new Pay(
        PlatformAddress.fromAccountId(
            "0x0000000000000000000000000000000000000000",
            { networkId: "tc" }
        ),
        new U64(11),
        "tc"
    );
    const signed = pay.sign({
        secret:
            "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd",
        seq: 0,
        fee: 0
    });
    expect(signed.hash()).toEqual(
        new H256(
            "6547527d42f407352b8d23470322e09261d6dee6fda43c10aa2f59aafa70ba4b"
        )
    );
});

test("toJSON", () => {
    const p = new Pay(
        PlatformAddress.fromAccountId(
            "0x0000000000000000000000000000000000000000",
            { networkId: "tc" }
        ),
        new U64(11),
        "tc"
    );
    p.setFee(33);
    p.setSeq(44);
    expect(fromJSONToTransaction(p.toJSON())).toEqual(p);
});
