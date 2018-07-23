import { readFileSync } from "fs";

describe.skip("examples", () => {
    beforeAll(() => {
        // import account "0xa6594b7196808d161b6fb137e781abbc251385d9" which is used in
        // the examples.
        // The passphrase of the account is "satoshi".
        runExample("import-test-account");
    });

    test("get-balance", () => {
        runExample("get-balance");
    });

    test("payment", () => {
        runExample("payment");
    });

    test("create-asset-transfer-address", () => {
        runExample("create-asset-transfer-address");
    });

    test("mint-and-transfer", () => {
        runExample("mint-and-transfer");
    });
});

// FIXME: The tests don't fail even if eval prints console.error
function runExample(name) {
    const path = `examples/${name}.js`;
    eval(String(readFileSync(path)).replace(`require("codechain-sdk")`, `require("..")`));
}
