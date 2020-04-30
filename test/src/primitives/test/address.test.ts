import { H256, Address } from "../src";
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

describe("fromString", () => {
    const pubkey =
        "7b5e0ee8644c6f585fc297364143280a458445085fc297364143280a45844502";
    const mainnetAddress =
        "ur5hx1u1e14O6GRMb1hfwpc2QUMoCkWERQhfwpc2QUMoCkWERQIcc0";
    const testnetAddress =
        "sc5hx1u1e14O6GRMb1hfwpc2QUMoCkWERQhfwpc2QUMoCkWERQItc0";

    it("mainnet", () => {
        const address = Address.fromString(mainnetAddress);
        expect(address.pubkey.value).equal(new H256(pubkey).value);
    });

    it("testnet", () => {
        const address = Address.fromString(testnetAddress);
        expect(address.pubkey.value).equal(new H256(pubkey).value);
    });

    it("Invalid checksum", () => {
        const invalidChecksumAddress =
            "sc5hz1u1e14O6GRMb1hfwpc2QUMoCkWERQhfwpc2QUMoCkWERQIcc0";
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

    it("mainnet", () => {
        const address = Address.fromPublic(pubkey, { networkId: "cc" });
        expect(address.toString()).equal(
            "2xsv1ngs16bSZoN8HFkTg7kNg1Boue1Y3TvOvW4oWRH1jkDOQTwcc0"
        );
        expect(address.pubkey).deep.equal(new H256(pubkey));
    });

    it("testnet", () => {
        const address = Address.fromPublic(pubkey, { networkId: "tc" });
        expect(address.toString()).equal(
            "01sv1ngs16bSZoN8HFkTg7kNg1Boue1Y3TvOvW4oWRH1jkDOQTwtc0"
        );
        expect(address.pubkey).deep.equal(new H256(pubkey));
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
