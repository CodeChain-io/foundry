import { execFile } from "child_process";
import { readFileSync, unlinkSync, writeFileSync } from "fs";

describe("examples", () => {
    beforeAll(done => {
        // import account "tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd" which is used in
        // the examples.
        // The passphrase of the account is "satoshi".
        runExample("import-test-account", done);
    });

    test("create-account-with-rpc", done => {
        runExample("create-account-with-rpc", done);
    });

    test("create-account-with-secret", done => {
        runExample("create-account-with-secret", done);
    });

    test("get-balance", done => {
        runExample("get-balance", done);
    });

    test("send-parcel", done => {
        runExample("send-parcel", done);
    });

    test("send-signed-parcel", done => {
        runExample("send-signed-parcel", done);
    });

    test("create-asset-transfer-address", done => {
        runExample("create-asset-transfer-address", done);
    });

    test("mint-asset", done => {
        runExample("mint-asset", done);
    });

    test("mint-and-transfer", done => {
        runExample("mint-and-transfer", done);
    });

    test("mint-and-burn", done => {
        runExample("mint-and-burn", done);
    });

    test("set-regular-key", done => {
        runExample("set-regular-key", done);
    });

    test("mint-and-compose", done => {
        runExample("mint-and-compose", done);
    });
});

function runExample(name: string, done: () => any) {
    const originalPath = `examples/${name}.js`;
    const code = String(readFileSync(originalPath)).replace(
        `require("codechain-sdk")`,
        `require("..")`
    );
    const testPath = `examples/test-${name}.js`;
    writeFileSync(testPath, code);
    execFile("node", [testPath], (_error, _stdout, stderr) => {
        expect(stderr).toBe("");
        unlinkSync(testPath);
        done();
    });
}
