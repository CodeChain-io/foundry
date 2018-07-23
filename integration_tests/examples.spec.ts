describe("examples", () => {
    beforeAll(() => {
        // import account "0xa6594b7196808d161b6fb137e781abbc251385d9" which is used in
        // the examples.
        // The passphrase of the account is "satoshi".
        require("../examples/import-test-account");
    });

    test("get-balance", () => {
        require("../examples/get-balance");
    });

    test("payment", () => {
        require("../examples/payment");
    });

    test("create-asset-transfer-address", () => {
        require("../examples/create-asset-transfer-address");
    });

    test("mint-and-transfer", () => {
        require("../examples/mint-and-transfer");
    });
});
