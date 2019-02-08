import { execFile } from "child_process";
import { readdirSync, readFileSync, unlinkSync, writeFileSync } from "fs";
import * as _ from "lodash";

describe("examples", () => {
    beforeAll(done => {
        // import account "tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd" which is used in
        // the examples.
        // The passphrase of the account is "satoshi".
        runExample("import-test-account", done);
    });

    const excludes = [
        "import-test-account",
        "mint-and-compose",
        "mint-and-compose-and-decompose"
    ];
    const tests: string[] = readdirSync("examples/")
        .filter(filename => filename.endsWith(".js"))
        .map(filename => filename.replace(/.js$/, ""))
        .filter(filename => !filename.startsWith("test-"))
        .filter(filename => !excludes.includes(filename));

    _.shuffle(tests).map(name =>
        test(name, (done: () => any) => runExample(name, done))
    );
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
