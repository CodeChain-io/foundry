import { SDK } from "../";

const SERVER_URL = "http://localhost:8080";
const sdk = new SDK(SERVER_URL);

test("getBlockNumber", async () => {
    const blockNumber = await sdk.getBlockNumber();
    expect(typeof blockNumber).toBe("number");
});
