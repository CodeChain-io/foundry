import { Invoice } from "../Invoice";

test("toJSON", () => {
    const invoice = new Invoice(true);
    expect(Invoice.fromJSON(invoice.toJSON())).toEqual(invoice);
});
