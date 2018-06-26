import { PlatformAddress } from "../PlatformAddress";
import { H160 } from "../primitives/H160";

test("PlatformAddress.fromAccountId - mainnet (default)", () => {
    const accountId = new H160("7b5e0ee8644c6f585fc297364143280a45844502");
    const address = PlatformAddress.fromAccountId(accountId);
    expect(address.value).toMatch(/^ccc[a-z0-9]+$/);
});

test("PlatformAddress.fromAccountId - testnet", () => {
    const accountId = new H160("7b5e0ee8644c6f585fc297364143280a45844502");
    const address = PlatformAddress.fromAccountId(accountId, { isTestnet: true });
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

test("PlatformAddress.fromAddress - mainnet", () => {
    const address = PlatformAddress.fromAddress("cccqpa4urhgv3xx7kzlc2tnvs2r9q9ytpz9qg4aw2qm");
    expect(address.accountId).toEqual(new H160("7b5e0ee8644c6f585fc297364143280a45844502"));
});

test("PlatformAddress.fromAddress - testnet", () => {
    const address = PlatformAddress.fromAddress("tccqpa4urhgv3xx7kzlc2tnvs2r9q9ytpz9qga6ufnz");
    expect(address.accountId).toEqual(new H160("7b5e0ee8644c6f585fc297364143280a45844502"));
});

test("PlatformAddress.fromAddress - invalid checksum", () => {
    expect(() => {
        PlatformAddress.fromAddress("cccqpa4urhgv3xx7kzlc2tnvs2r9q9ytpz9qgqqqqqq");
    }).toThrow();
});
