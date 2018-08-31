import { H160 } from "../../core/H160";

import { PlatformAddress } from "../PlatformAddress";

describe("fromAccountId", () => {
    const accountId = new H160("7b5e0ee8644c6f585fc297364143280a45844502");

    test("mainnet", () => {
        const address = PlatformAddress.fromAccountId(accountId, {
            networkId: "cc"
        });
        expect(address.value).toMatch(/^ccc[a-z0-9]+$/);
    });

    test("testnet", () => {
        const address = PlatformAddress.fromAccountId(accountId);
        expect(address.value).toMatch(/^tcc[a-z0-9]+$/);
    });

    test("Valid version", () => {
        expect(() => {
            PlatformAddress.fromAccountId(accountId, { version: 0 });
        }).not.toThrow();
    });

    test("Invalid version", done => {
        try {
            PlatformAddress.fromAccountId(accountId, { version: 99 });
            done.fail();
        } catch (e) {
            expect(e.toString()).toContain("version");
            done();
        }
    });

    test("Invalid networkId", done => {
        try {
            PlatformAddress.fromAccountId(accountId, {
                version: 0,
                networkId: "x"
            });
            done.fail();
        } catch (e) {
            expect(e.toString()).toContain("networkId");
            expect(e.toString()).toContain("x");
            done();
        }
    });

    test("Invalid accountId", done => {
        try {
            PlatformAddress.fromAccountId("xxx");
            done.fail();
        } catch (e) {
            expect(e.toString()).toContain("accountId");
            expect(e.toString()).toContain("xxx");
            done();
        }
    });
});

describe("fromString", () => {
    const accountId = "7b5e0ee8644c6f585fc297364143280a45844502";
    const mainnetAddress = "cccqpa4urhgv3xx7kzlc2tnvs2r9q9ytpz9qg4aw2qm";
    const testnetAddress = "tccqpa4urhgv3xx7kzlc2tnvs2r9q9ytpz9qga6ufnz";

    test("mainnet", () => {
        const address = PlatformAddress.fromString(mainnetAddress);
        expect(address.accountId).toEqual(new H160(accountId));
    });

    test("testnet", () => {
        const address = PlatformAddress.fromString(testnetAddress);
        expect(address.accountId).toEqual(new H160(accountId));
    });

    test("Invalid checksum", done => {
        const invalidChecksumAddress =
            "cccqpa4urhgv3xx7kzlc2tnvs2r9q9ytpz9qgqqqqqq";
        try {
            PlatformAddress.fromString(invalidChecksumAddress);
            done.fail();
        } catch (e) {
            expect(e.toString()).toContain("checksum");
            done();
        }
    });
});
