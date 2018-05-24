import { SDK, H256 } from "../";
import { payment } from "./helper";

const SERVER_URL = "http://localhost:8080";
const sdk = new SDK(SERVER_URL);

test("getParcelInvoice", async () => {
    const hash = await payment();
    const invoice = await sdk.getParcelInvoice(hash);
    expect(invoice).toEqual({ "outcome": "Success" });
});

test("getParcelInvoice - null", async () => {
    const hash = new H256("0000000000000000000000000000000000000000000000000000000000000000");
    const invoice = await sdk.getParcelInvoice(hash);
});
