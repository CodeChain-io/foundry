import { Invoice } from "../Invoice";

test("rlp", () => {
    expect(new Invoice(true).rlpBytes()).toEqual(Buffer.from([0x01]));
    expect(new Invoice(false).rlpBytes()).toEqual(Buffer.from([0x00]));

    expect(Invoice.fromBytes(Buffer.from([0x01]))).toEqual(new Invoice(true));
    expect(Invoice.fromBytes(Buffer.from([0x00]))).toEqual(new Invoice(false));
});