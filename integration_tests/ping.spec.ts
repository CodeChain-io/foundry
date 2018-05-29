import { SDK } from "../";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";

test("ping", async () => {
    const response = await new SDK(SERVER_URL).ping();
    expect(response).toBe("pong");
});
