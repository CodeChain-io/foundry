import { SDK, H160, H512 } from "..";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK(SERVER_URL);

test.skip("getRegularKey", async () => {
    const address = new H160("a6594b7196808d161b6fb137e781abbc251385d9");
    const regularKey = await sdk.getRegularKey(address);
    expect(regularKey).toEqual(expect.any(H512));
});
