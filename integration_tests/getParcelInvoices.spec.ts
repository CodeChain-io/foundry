import { Invoice, SDK, H256 } from "../";
import { payment } from "./helper";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK(SERVER_URL);

test("getParcelInvoice", async () => {
    const hash = await payment();
    const invoice = await sdk.getParcelInvoices(hash);
    expect(invoice).toEqual(new Invoice(true));
});

test("getParcelInvoices - null", async () => {
    const hash = new H256("0000000000000000000000000000000000000000000000000000000000000000");
    const invoice = await sdk.getParcelInvoices(hash);
    expect(invoice).toBe(null);
});
