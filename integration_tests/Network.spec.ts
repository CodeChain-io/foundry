import { SDK } from "../";
import { SERVER_URL } from "./helper";

describe("network", () => {
    let sdk: SDK;

    beforeAll(async () => {
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
