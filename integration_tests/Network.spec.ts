import { SDK } from "../";

describe("network", () => {
    let sdk: SDK;

    beforeAll(async () => {
        const SERVER_URL =
            process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
        sdk = new SDK({ server: SERVER_URL });
    });

    test("whitelist enabled", async () => {
        const { enabled } = await sdk.rpc.network.getWhitelist();
        expect(enabled).toBe(false);
    });

    test("blacklist enabled", async () => {
        const { enabled } = await sdk.rpc.network.getBlacklist();
        expect(enabled).toBe(false);
    });
});
