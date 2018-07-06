import { SDK } from "../";
import { payment } from "./helper";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK({ server: SERVER_URL });

test("getParcelInvoice", async () => {
    const { Invoice } = SDK.Core.classes;
    const hash = await payment();
    const invoice = await sdk.rpc.chain.getParcelInvoice(hash);
    expect(invoice).toEqual(new Invoice(true));
});

test("getParcelInvoice - null", async () => {
    const hash = "0x0000000000000000000000000000000000000000000000000000000000000000";
    const invoice = await sdk.rpc.chain.getParcelInvoice(hash);
    expect(invoice).toBe(null);
});
