import { H160 } from "../../core/H160";

import { PlatformAddress } from "../PlatformAddress";

test("PlatformAddress.fromAccountId - mainnet (default)", () => {
    const accountId = new H160("7b5e0ee8644c6f585fc297364143280a45844502");
    const address = PlatformAddress.fromAccountId(accountId, {
        networkId: "cc"
    });
    expect(address.value).toMatch(/^ccc[a-z0-9]+$/);
});

test("PlatformAddress.fromAccountId - testnet", () => {
    const accountId = new H160("7b5e0ee8644c6f585fc297364143280a45844502");
    const address = PlatformAddress.fromAccountId(accountId);
    expect(address.value).toMatch(/^tcc[a-z0-9]+$/);
});

test("PlatformAddress.fromAccountId - valid version", () => {
    const accountId = new H160("7b5e0ee8644c6f585fc297364143280a45844502");
    expect(() => {
        PlatformAddress.fromAccountId(accountId, { version: 0 });
    }).not.toThrow();
});

test("PlatformAddress.fromAccountId - invalid version", () => {
    const accountId = new H160("7b5e0ee8644c6f585fc297364143280a45844502");
    expect(() => {
        PlatformAddress.fromAccountId(accountId, { version: 1 });
    }).toThrow("Unsupported version for platform address: 1");
});

test("PlatformAddress.fromString - mainnet", () => {
    const address = PlatformAddress.fromString(
        "cccqpa4urhgv3xx7kzlc2tnvs2r9q9ytpz9qg4aw2qm"
    );
    expect(address.accountId).toEqual(
        new H160("7b5e0ee8644c6f585fc297364143280a45844502")
    );
});

test("PlatformAddress.fromString - testnet", () => {
    const address = PlatformAddress.fromString(
        "tccqpa4urhgv3xx7kzlc2tnvs2r9q9ytpz9qga6ufnz"
    );
    expect(address.accountId).toEqual(
        new H160("7b5e0ee8644c6f585fc297364143280a45844502")
    );
});

test("PlatformAddress.fromString - invalid checksum", () => {
    expect(() => {
        PlatformAddress.fromString(
            "cccqpa4urhgv3xx7kzlc2tnvs2r9q9ytpz9qgqqqqqq"
        );
    }).toThrow();
});
