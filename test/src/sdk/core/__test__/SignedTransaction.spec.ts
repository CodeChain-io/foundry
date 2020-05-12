import { expect } from "chai";
import "mocha";
import { Address, H512, U64 } from "../../../primitives/src";
import { getPublicFromPrivate } from "../../utils";
import { Pay } from "../classes";
import { fromJSONToSignedTransaction } from "../transaction/json";

it("toJSON", () => {
    const secret = new H512(
        "9af28f6fd6a1170dbee2cb8c34abab0408e6d811d212cdcde23f72473eb0d97ad7a6d266837c1c591383b90d835068b9ed58dd3bcebd6e285911f58e40ce413c"
    );
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
    const p = pay.sign({
        secret,
        fee: 33,
        seq: 33
    });
    expect(fromJSONToSignedTransaction(p.toJSON())).deep.equal(p);
});

it("getSignerPublic", () => {
    const secret = new H512(
        "9af28f6fd6a1170dbee2cb8c34abab0408e6d811d212cdcde23f72473eb0d97ad7a6d266837c1c591383b90d835068b9ed58dd3bcebd6e285911f58e40ce413c"
    );
    const signerPubKey = Address.fromPublic(
        getPublicFromPrivate(secret.value),
        { networkId: "tc" }
    ).getPubKey();
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
    const p = pay.sign({
        secret,
        fee: 33,
        seq: 44
    });
    expect(p.getSignerPublic().value).equal(signerPubKey.value);
});

it("getSignerAddress", () => {
    const secret = new H512(
        "9af28f6fd6a1170dbee2cb8c34abab0408e6d811d212cdcde23f72473eb0d97ad7a6d266837c1c591383b90d835068b9ed58dd3bcebd6e285911f58e40ce413c"
    );
    const signerPubKey = Address.fromPublic(
        getPublicFromPrivate(secret.value),
        { networkId: "tc" }
    ).getPubKey();
    const signerAddress = Address.fromPublic(signerPubKey, {
        networkId: "tc"
    });
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
    const p = pay.sign({
        secret,
        fee: 33,
        seq: 44
    });
    expect(p.getSignerAddress({ networkId: "tc" }).value).equal(
        signerAddress.value
    );
});
