test("commonjs", async () => {
    const obj = require("../");
    expect(obj).toMatchObject({
        SDK: expect.any(Function),
    });
});
