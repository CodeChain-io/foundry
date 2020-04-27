import { H160, Address } from "../src";
import "mocha";
import { expect } from "chai";

describe("Address", () => {
    it("import", () => {
        expect(typeof Address).equal("function");
    });

    it("require", () => {
        const obj = require("../src");
        expect(typeof obj.Address).equal("function");
    });

    it.skip("check", () => expect.fail("not implemented"));
    it.skip("ensure", () => expect.fail("not implemented"));
    it.skip("ensureAccount", () => expect.fail("not implemented"));
    it.skip("toString", () => expect.fail("not implemented"));
});

describe("fromAccountId", () => {
    const accountId = new H160("7b5e0ee8644c6f585fc297364143280a45844502");

    it("mainnet", () => {
        const address = Address.fromAccountId(accountId, {
            networkId: "cc"
        });
        expect(address.value).equal(
            "cccq9a4urhgv3xx7kzlc2tnvs2r9q9ytpz9qgs7q0a7"
        );
    });

    it("testnet", () => {
        const address = Address.fromAccountId(accountId, {
            networkId: "tc"
        });
        expect(address.value).equal(
            "tccq9a4urhgv3xx7kzlc2tnvs2r9q9ytpz9qgcejvw8"
        );
    });

    it("Valid version", () => {
        expect(() => {
            Address.fromAccountId(accountId, {
                networkId: "tc",
                version: 1
            });
        }).not.throw();
    });

    it("Invalid version", () => {
        try {
            Address.fromAccountId(accountId, {
                networkId: "tc",
                version: 99
            });
            expect.fail();
        } catch (e) {
            expect(e.toString()).contains("version");
        }
    });

    it("Invalid networkId", () => {
        try {
            Address.fromAccountId(accountId, {
                version: 1,
                networkId: "x"
            });
            expect.fail();
        } catch (e) {
            expect(e.toString()).contains("networkId");
            expect(e.toString()).contains("x");
        }
    });

    it("Invalid accountId", () => {
        try {
            Address.fromAccountId("xxx", { networkId: "tc" });
            expect.fail();
        } catch (e) {
            expect(e.toString()).contains("accountId");
            expect(e.toString()).contains("xxx");
        }
    });
});

describe("fromString", () => {
    const accountId = "7b5e0ee8644c6f585fc297364143280a45844502";
    const mainnetAddress = "cccq9a4urhgv3xx7kzlc2tnvs2r9q9ytpz9qgs7q0a7";
    const testnetAddress = "tccq9a4urhgv3xx7kzlc2tnvs2r9q9ytpz9qgcejvw8";

    it("mainnet", () => {
        const address = Address.fromString(mainnetAddress);
        expect(address.accountId.value).equal(new H160(accountId).value);
    });

    it("testnet", () => {
        const address = Address.fromString(testnetAddress);
        expect(address.accountId.value).equal(new H160(accountId).value);
    });

    it("Invalid checksum", () => {
        const invalidChecksumAddress =
            "cccqpa4urhgv3xx7kzlc2tnvs2r9q9ytpz9qgqqqqqq";
        try {
            Address.fromString(invalidChecksumAddress);
            expect.fail();
        } catch (e) {
            expect(e.toString()).contains("checksum");
        }
    });
});

describe("fromPublic", () => {
    const pubkey =
        "d7a6d266837c1c591383b90d835068b9ed58dd3bcebd6e285911f58e40ce413c";
    const accountId = "837cfc9c54fd1cd83970e0493d54d3a579aba06c";

    it("mainnet", () => {
        const address = Address.fromPublic(pubkey, { networkId: "cc" });
        expect(address.toString()).equal(
            "cccqxphelyu2n73ekpewrsyj0256wjhn2aqdsdp3qs3"
        );
        expect(address.accountId).deep.equal(new H160(accountId));
    });

    it("testnet", () => {
        const address = Address.fromPublic(pubkey, { networkId: "tc" });
        expect(address.toString()).equal(
            "tccqxphelyu2n73ekpewrsyj0256wjhn2aqds9xrrrg"
        );
        expect(address.accountId).deep.equal(new H160(accountId));
    });

    it("Invalid public key", () => {
        try {
            Address.fromPublic(pubkey.slice(1), { networkId: "cc" });
            expect.fail();
        } catch (e) {
            expect(e.toString()).contains("Invalid public key");
        }
    });
});
