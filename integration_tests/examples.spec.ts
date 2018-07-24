import { readFileSync } from "fs";

describe.skip("examples", () => {
    beforeAll(() => {
        // import account "cccqzn9jjm3j6qg69smd7cn0eup4w7z2yu9myd6c4d7" which is used in
        // the examples.
        // The passphrase of the account is "satoshi".
        runExample("import-test-account");
    });

    test("create-account", () => {
        runExample("create-account");
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

    test("mint-asset", () => {
        runExample("mint-asset");
    });

    test.skip("mint-and-transfer", () => {
        runExample("mint-and-transfer");
    });
});

// FIXME: The tests don't fail even if eval prints console.error
function runExample(name) {
    const path = `examples/${name}.js`;
    eval(String(readFileSync(path)).replace(`require("codechain-sdk")`, `require("..")`));
}
