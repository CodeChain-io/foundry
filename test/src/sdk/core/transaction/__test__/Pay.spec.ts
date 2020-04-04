import { expect } from "chai";
import { Address, H256, U64 } from "foundry-primitives";
import "mocha";
import { Pay } from "../Pay";

import { fromJSONToTransaction } from "../json";

it("rlp", () => {
    const t = new Pay(
        Address.fromAccountId("0x0000000000000000000000000000000000000000", {
            networkId: "tc"
        }),
        new U64(11),
        "tc"
    );
    t.setFee(0);
    t.setSeq(0);
    expect(t.rlpBytes()).deep.equal(
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

it("hash", () => {
    const t = new Pay(
        Address.fromAccountId("0x0000000000000000000000000000000000000000", {
            networkId: "tc"
        }),
        new U64(11),
        "tc"
    );
    t.setFee(0);
    t.setSeq(0);
    expect(t.unsignedHash().value).equal(
        "3b578bebb32cae770ab1094d572a4721b624fc101bb88fbc580eeb2931f65665"
    );
});

it("sign", () => {
    const pay = new Pay(
        Address.fromAccountId("0x0000000000000000000000000000000000000000", {
            networkId: "tc"
        }),
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
        "463ffba7f7f1527d66e9769204bbfec902e6bfaa8cfc9c161aad72bd6db25a1b97c78364fbade7ff6fa1a36e1da9205b646cdcefe82636f756bead33aa1e8a04"
    );
});

it("signed hash", () => {
    const pay = new Pay(
        Address.fromAccountId("0x0000000000000000000000000000000000000000", {
            networkId: "tc"
        }),
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
        "21b9c5dda38bee928e6619f7b7266bfee120aadb62ed4f7eceb2893f2ee7c76e"
    );
});

it("toJSON", () => {
    const p = new Pay(
        Address.fromAccountId("0x0000000000000000000000000000000000000000", {
            networkId: "tc"
        }),
        new U64(11),
        "tc"
    );
    p.setFee(33);
    p.setSeq(44);
    expect(fromJSONToTransaction(p.toJSON())).deep.equal(p);
});
