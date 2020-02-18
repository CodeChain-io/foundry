import { SDK } from "../";

test("import", () => {
    expect(() => {
        new SDK({ server: "http://localhost:8080" });
    }).not.toThrow();
});

test("require", () => {
    const CodeChainSdk = require("../");
    expect(
        () => new CodeChainSdk({ server: "http://localhost:8080" })
    ).not.toThrow();
});
