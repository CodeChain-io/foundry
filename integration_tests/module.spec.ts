import { SDK } from "../";

test("import", () => {
    expect(() => {
        new SDK({ server: "http://localhost:8080" });
    }).not.toThrow();
});

test("require", () => {
    const CodeChainSdk = require("../");
    expect(() => {
        const sdk = new CodeChainSdk({ server: "http://localhost:8080" });
    }).not.toThrow();
});
