import { SDK } from "../";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";

test("getNodeVersion", async () => {
    const response = await new SDK({ server: SERVER_URL }).rpc.node.getNodeVersion();
    expect(response).toBe("0.1.0");
});
