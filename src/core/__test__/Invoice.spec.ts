import { Invoice } from "../Invoice";

test("toJSON", () => {
    const invoice = new Invoice(true);
    expect(Invoice.fromJSON(invoice.toJSON())).toEqual(invoice);
});

describe("fromJSON", () => {
    test("success", () => {
        const json = {
            success: true
        };
        expect(Invoice.fromJSON(json)).toEqual(new Invoice(true));
    });

    describe("error", () => {
        test("type only", () => {
            const json = {
                success: false,
                error: {
                    type: "InvalidScript"
                }
            };
            expect(Invoice.fromJSON(json)).toEqual(new Invoice(false, { type: "InvalidScript" }));
        });

        test("string content", () => {
            const json = {
                success: false,
                error: {
                    type: "AssetNotFound",
                    content: "0x0000000000000000000000000000000000000000000000000000000000000000"
                }
            };
            expect(Invoice.fromJSON(json)).toEqual(new Invoice(false, json.error));
        });

        test("object content", () => {
            const json = {
                success: false,
                error: {
                    type: "InvalidAssetAmount",
                    content: {
                        address: "0x0000000000000000000000000000000000000000000000000000000000000000",
                        expected: 0,
                        got: 1
                    }
                }
            };
            expect(Invoice.fromJSON(json)).toEqual(new Invoice(false, json.error));
        });
    });
});
