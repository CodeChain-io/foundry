describe("examples", () => {
    beforeAll(() => {
        // import account "0xa6594b7196808d161b6fb137e781abbc251385d9" which is used in
        // the examples.
        // The passphrase of the account is "satoshi".
        require("../examples/import-test-account");
    });

    test("payment", () => {
        require("../examples/payment");
    });

    test("mint-and-transfer", () => {
        require("../examples/mint-and-transfer");
    });
});
