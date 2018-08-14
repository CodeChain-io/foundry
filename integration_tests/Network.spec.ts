import { SDK } from "../";

describe("network", () => {
    let sdk: SDK;

    beforeAll(async () => {
        sdk = new SDK({ server: "http://localhost:8080" });
    });

    test("whitelist enabled", async () => {
        let { enabled } = await sdk.rpc.network.getWhitelist();
        expect(enabled).toBe(false);
    });

    test("blacklist enabled", async () => {
        let { enabled } = await sdk.rpc.network.getBlacklist();
        expect(enabled).toBe(false);
    });
});
