import { PlatformAddress } from "codechain-primitives";

import { H256 } from "../H256";
import { U256 } from "../U256";
import { Parcel } from "../Parcel";
import { SignedParcel } from "../SignedParcel";
import { getAccountIdFromPrivate } from "../../utils";

test("toJSON", () => {
    const secret = new H256(
        "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd"
    );
    const p = Parcel.payment(
        "tc",
        PlatformAddress.fromAccountId(
            "0x0000000000000000000000000000000000000000"
        ),
        new U256(11)
    ).sign({
        secret,
        fee: 33,
        nonce: 33
    });
    expect(SignedParcel.fromJSON(p.toJSON())).toEqual(p);
});

test("getSignerAccountId", () => {
    const secret = new H256(
        "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd"
    );
    const signerAccountId = PlatformAddress.fromAccountId(
        getAccountIdFromPrivate(secret.value)
    ).getAccountId();
    const p = Parcel.payment(
        "tc",
        PlatformAddress.fromAccountId(
            "0x0000000000000000000000000000000000000000"
        ),
        new U256(11)
    ).sign({
        secret,
        fee: 33,
        nonce: 44
    });
    expect(p.getSignerAccountId()).toEqual(signerAccountId);
});

test("getSignerAddress", () => {
    const secret = new H256(
        "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd"
    );
    const signerAccountId = PlatformAddress.fromAccountId(
        getAccountIdFromPrivate(secret.value)
    ).getAccountId();
    const signerAddress = PlatformAddress.fromAccountId(signerAccountId);
    const p = Parcel.payment(
        "tc",
        PlatformAddress.fromAccountId(
            "0x0000000000000000000000000000000000000000"
        ),
        new U256(11)
    ).sign({
        secret,
        fee: 33,
        nonce: 44
    });
    expect(p.getSignerAddress()).toEqual(signerAddress);
});
