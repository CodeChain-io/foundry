import { PlatformAddress } from "codechain-primitives";

import { H256 } from "../H256";
import { Parcel } from "../Parcel";
import { U256 } from "../U256";
import { U64 } from "../U64";

test("rlp", () => {
    const t = Parcel.payment(
        "tc",
        PlatformAddress.fromAccountId(
            "0x0000000000000000000000000000000000000000",
            { networkId: "tc" }
        ),
        new U64(11)
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
    const t = Parcel.payment(
        "tc",
        PlatformAddress.fromAccountId(
            "0x0000000000000000000000000000000000000000",
            { networkId: "tc" }
        ),
        new U64(11)
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
    const t = Parcel.payment(
        "tc",
        PlatformAddress.fromAccountId(
            "0x0000000000000000000000000000000000000000",
            { networkId: "tc" }
        ),
        new U64(11)
    );
    const signed = t.sign({
        secret:
            "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd",
        seq: 0,
        fee: 0
    });
    const { v, r, s } = signed.signature();
    expect(v).toBe(1);
    expect(r.toEncodeObject()).toEqual(
        new U256(
            "0x3f9bcff484bd5f1d5549f912f9eeaf8c2fe349b257bde2b61fb1036013d4e44c"
        ).toEncodeObject()
    );
    expect(s.toEncodeObject()).toEqual(
        new U256(
            "0x204a4215d26cb879eaad2028fe1a7898e4cf9a5d979eb383e0a384140d6e04c1"
        ).toEncodeObject()
    );
});

test("signed hash", () => {
    const t = Parcel.payment(
        "tc",
        PlatformAddress.fromAccountId(
            "0x0000000000000000000000000000000000000000",
            { networkId: "tc" }
        ),
        new U64(11)
    );
    const signed = t.sign({
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
    const p = Parcel.payment(
        "tc",
        PlatformAddress.fromAccountId(
            "0x0000000000000000000000000000000000000000",
            { networkId: "tc" }
        ),
        new U64(11)
    );
    p.setFee(33);
    p.setSeq(44);
    expect(Parcel.fromJSON(p.toJSON())).toEqual(p);
});
