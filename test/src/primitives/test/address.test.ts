import { H160, Address } from "../";

describe("Address", () => {
    test("import", () => {
        expect(typeof Address).toBe("function");
    });

    test("require", () => {
        const obj = require("..");
        expect(typeof obj.Address).toBe("function");
    });

    test.skip("check", done => done.fail("not implemented"));
    test.skip("ensure", done => done.fail("not implemented"));
    test.skip("ensureAccount", done => done.fail("not implemented"));
    test.skip("toString", done => done.fail("not implemented"));
});

describe("fromAccountId", () => {
    const accountId = new H160("7b5e0ee8644c6f585fc297364143280a45844502");

    test("mainnet", () => {
        const address = Address.fromAccountId(accountId, {
            networkId: "cc"
        });
        expect(address.value).toBe(
            "cccq9a4urhgv3xx7kzlc2tnvs2r9q9ytpz9qgs7q0a7"
        );
    });

    test("testnet", () => {
        const address = Address.fromAccountId(accountId, {
            networkId: "tc"
        });
        expect(address.value).toBe(
            "tccq9a4urhgv3xx7kzlc2tnvs2r9q9ytpz9qgcejvw8"
        );
    });

    test("Valid version", () => {
        expect(() => {
            Address.fromAccountId(accountId, {
                networkId: "tc",
                version: 1
            });
        }).not.toThrow();
    });

    test("Invalid version", done => {
        try {
            Address.fromAccountId(accountId, {
                networkId: "tc",
                version: 99
            });
            done.fail();
        } catch (e) {
            expect(e.toString()).toContain("version");
            done();
        }
    });

    test("Invalid networkId", done => {
        try {
            Address.fromAccountId(accountId, {
                version: 1,
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
            Address.fromAccountId("xxx", { networkId: "tc" });
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
    const mainnetAddress = "cccq9a4urhgv3xx7kzlc2tnvs2r9q9ytpz9qgs7q0a7";
    const testnetAddress = "tccq9a4urhgv3xx7kzlc2tnvs2r9q9ytpz9qgcejvw8";

    test("mainnet", () => {
        const address = Address.fromString(mainnetAddress);
        expect(address.accountId).toEqual(new H160(accountId));
    });

    test("testnet", () => {
        const address = Address.fromString(testnetAddress);
        expect(address.accountId).toEqual(new H160(accountId));
    });

    test("Invalid checksum", done => {
        const invalidChecksumAddress =
            "cccqpa4urhgv3xx7kzlc2tnvs2r9q9ytpz9qgqqqqqq";
        try {
            Address.fromString(invalidChecksumAddress);
            done.fail();
        } catch (e) {
            expect(e.toString()).toContain("checksum");
            done();
        }
    });
});

describe("fromPublic", () => {
    const pubkey =
        "d7a6d266837c1c591383b90d835068b9ed58dd3bcebd6e285911f58e40ce413c";
    const accountId = "837cfc9c54fd1cd83970e0493d54d3a579aba06c";

    test("mainnet", () => {
        const address = Address.fromPublic(pubkey, { networkId: "cc" });
        expect(address.toString()).toEqual(
            "cccqxphelyu2n73ekpewrsyj0256wjhn2aqdsdp3qs3"
        );
        expect(address.accountId).toEqual(new H160(accountId));
    });

    test("testnet", () => {
        const address = Address.fromPublic(pubkey, { networkId: "tc" });
        expect(address.toString()).toEqual(
            "tccqxphelyu2n73ekpewrsyj0256wjhn2aqds9xrrrg"
        );
        expect(address.accountId).toEqual(new H160(accountId));
    });

    test("Invalid public key", done => {
        try {
            Address.fromPublic(pubkey.slice(1), { networkId: "cc" });
            done.fail();
        } catch (e) {
            expect(e.toString()).toContain("Invalid public key");
            done();
        }
    });
});
