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

    test("Invalid version", () => {
        expect(() => {
            PlatformAddress.fromAccountId(accountId, { version: 1 });
        }).toThrow("Unsupported version for platform address: 1");
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

    test("Invalid checksum", () => {
        const invalidChecksumAddress =
            "cccqpa4urhgv3xx7kzlc2tnvs2r9q9ytpz9qgqqqqqq";
        try {
            PlatformAddress.fromString(invalidChecksumAddress);
        } catch (e) {
            expect(e.toString()).toContain("checksum");
        }
    });
});
