import { readFileSync, writeFileSync, unlinkSync } from "fs";
import { execFile } from "child_process";

describe("examples", () => {
    beforeAll((done) => {
        // import account "cccqzn9jjm3j6qg69smd7cn0eup4w7z2yu9myd6c4d7" which is used in
        // the examples.
        // The passphrase of the account is "satoshi".
        runExample("import-test-account", done);
    });

    test("create-account", (done) => {
        runExample("create-account", done);
    });

    test("get-balance", (done) => {
        runExample("get-balance", done);
    });

    test("payment", (done) => {
        runExample("payment", done);
    });

    test("create-asset-transfer-address", (done) => {
        runExample("create-asset-transfer-address", done);
    });

    test("mint-asset", (done) => {
        runExample("mint-asset", done);
    });

    test("mint-and-transfer", (done) => {
        runExample("mint-and-transfer", done);
    });

    test("set-regular-key", (done) => {
        runExample("set-regular-key", done);
    });
});

function runExample(name, done) {
    const originalPath = `examples/${name}.js`;
    const code = String(readFileSync(originalPath)).replace(`require("codechain-sdk")`, `require("..")`);
    const testPath = `examples/test-${name}.js`;
    writeFileSync(testPath, code);
    execFile("node", [testPath], (error, stdout, stderr) => {
        expect(stderr).toBe("");
        unlinkSync(testPath);
        done();
    });
}
