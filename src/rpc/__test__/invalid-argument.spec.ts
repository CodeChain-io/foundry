import { Rpc } from "..";

describe("Invalid function argument", () => {
    const rpc = new Rpc({ server: "" });

    describe("getRegularKeyOwner", () => {
        const regularKey =
            "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        const invalidRegularKey = "0x0";
        const invalidBlockNumber: any = null;

        test("first argument", () => {
            expect(() => {
                rpc.chain.getRegularKeyOwner(invalidRegularKey);
            }).toThrow(invalidRegularKey);
        });

        test("second argument", () => {
            expect(() => {
                rpc.chain.getRegularKeyOwner(regularKey, invalidBlockNumber);
            }).toThrow("null");
        });
    });
});
