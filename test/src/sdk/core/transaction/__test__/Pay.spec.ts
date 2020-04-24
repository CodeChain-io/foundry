import { expect } from "chai";
import "mocha";
import { Address, U64 } from "../../../../primitives/src";
import { Pay } from "../Pay";

import { fromJSONToTransaction } from "../json";

it("rlp", () => {
    const t = new Pay(
        Address.fromPublic(
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            {
                networkId: "tc"
            }
        ),
        new U64(11),
        "tc"
    );
    t.setFee(0);
    t.setSeq(0);
    expect(t.rlpBytes()).deep.equal(
        Buffer.from([
            233,
            128,
            128,
            130,
            116,
            99,
            227,
            2,
            160,
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

it("hash", () => {
    const t = new Pay(
        Address.fromPublic(
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            {
                networkId: "tc"
            }
        ),
        new U64(11),
        "tc"
    );
    t.setFee(0);
    t.setSeq(0);
    expect(t.unsignedHash().value).equal(
        "9e30ad5042770e2ccc282c99736760b5b438b967bfa70afa424bae6f84aa2230"
    );
});

it("sign", () => {
    const pay = new Pay(
        Address.fromPublic(
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            {
                networkId: "tc"
            }
        ),
        new U64(11),
        "tc"
    );
    const signed = pay.sign({
        secret:
            "9af28f6fd6a1170dbee2cb8c34abab0408e6d811d212cdcde23f72473eb0d97ad7a6d266837c1c591383b90d835068b9ed58dd3bcebd6e285911f58e40ce413c",
        seq: 0,
        fee: 0
    });
    expect(signed.signature()).equal(
        "c0ccdc2c4966308d50f9e7e8c41078094c4cc3f2271f8d6e19e570b5afcb1098c2766202aec330d9f6d3c56158b6671dfc32697b08f2758654f80a615520370d"
    );
});

it("signed hash", () => {
    const pay = new Pay(
        Address.fromPublic(
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            {
                networkId: "tc"
            }
        ),
        new U64(11),
        "tc"
    );
    const signed = pay.sign({
        secret:
            "9af28f6fd6a1170dbee2cb8c34abab0408e6d811d212cdcde23f72473eb0d97ad7a6d266837c1c591383b90d835068b9ed58dd3bcebd6e285911f58e40ce413c",
        seq: 0,
        fee: 0
    });
    expect(signed.hash().value).equal(
        "7e3c89734e16ed7c173fdeba026f65f53aebefe9f8caf1feda5d2f96ffd916b6"
    );
});

it("toJSON", () => {
    const p = new Pay(
        Address.fromPublic(
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            {
                networkId: "tc"
            }
        ),
        new U64(11),
        "tc"
    );
    p.setFee(33);
    p.setSeq(44);
    expect(fromJSONToTransaction(p.toJSON())).deep.equal(p);
});
