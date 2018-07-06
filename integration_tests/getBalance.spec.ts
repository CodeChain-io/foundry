import { SDK } from "../";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK({ server: SERVER_URL });

test("getBalance", async () => {
    const address = "0xa6594b7196808d161b6fb137e781abbc251385d9";
    const balance = await sdk.rpc.chain.getBalance(address);
    expect(balance.value).toBeTruthy();
});
