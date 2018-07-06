test("commonjs", async () => {
    const CodeChainSdk = require("../");
    expect(() => {
        const sdk = new CodeChainSdk({ server: "http://localhost:8080" });
    }).not.toThrow();
});
