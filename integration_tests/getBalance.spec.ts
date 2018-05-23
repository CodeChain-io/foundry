import { SDK, H160, U256 } from "../";

const SERVER_URL = "http://localhost:8080";
const sdk = new SDK(SERVER_URL);

test("getBalance", async () => {
    const address = new H160("a6594b7196808d161b6fb137e781abbc251385d9");
    const balance = await sdk.getBalance(address);
    expect(balance instanceof U256).toBeTruthy();
});
