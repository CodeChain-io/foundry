import { PlatformAddress } from "codechain-primitives";

import { getAccountIdFromPrivate } from "../../utils";
import { Pay } from "../classes";
import { H256 } from "../H256";
import { fromJSONToSignedTransaction } from "../transaction/json";
import { U64 } from "../U64";

test("toJSON", () => {
    const secret = new H256(
        "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd"
    );
    const pay = new Pay(
        PlatformAddress.fromAccountId(
            "0x0000000000000000000000000000000000000000",
            { networkId: "tc" }
        ),
        new U64(11),
        "tc"
    );
    const p = pay.sign({
        secret,
        fee: 33,
        seq: 33
    });
    expect(fromJSONToSignedTransaction(p.toJSON())).toEqual(p);
});

test("getSignerAccountId", () => {
    const secret = new H256(
        "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd"
    );
    const signerAccountId = PlatformAddress.fromAccountId(
        getAccountIdFromPrivate(secret.value),
        { networkId: "tc" }
    ).getAccountId();
    const pay = new Pay(
        PlatformAddress.fromAccountId(
            "0x0000000000000000000000000000000000000000",
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
    expect(p.getSignerAccountId()).toEqual(signerAccountId);
});

test("getSignerAddress", () => {
    const secret = new H256(
        "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd"
    );
    const signerAccountId = PlatformAddress.fromAccountId(
        getAccountIdFromPrivate(secret.value),
        { networkId: "tc" }
    ).getAccountId();
    const signerAddress = PlatformAddress.fromAccountId(signerAccountId, {
        networkId: "tc"
    });
    const pay = new Pay(
        PlatformAddress.fromAccountId(
            "0x0000000000000000000000000000000000000000",
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
    expect(p.getSignerAddress({ networkId: "tc" })).toEqual(signerAddress);
});
