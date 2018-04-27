import { H256, U256, Action, Transaction } from "../index";

test("rlp", () => {
    const t = new Transaction(new U256(0), new U256(0), new Action("noop"), 1);
    expect(t.rlpBytes()).toEqual(Buffer.from([0xc4, 0x80, 0x80, 0x80, 0x01]));
});

test("hash", () => {
    const t = new Transaction(new U256(0), new U256(0), new Action("noop"), 1);
    expect(t.hash()).toEqual(new H256("828d02e6a218886c27505cd99cdcc64f22165ce166c4dce331f2dd27b50a4ecf"));
});

test("sign", () => {
    const t = new Transaction(new U256(0), new U256(0), new Action("noop"), 1);
    const signed = t.sign(new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd"));
    const { v, r, s } = signed.signature();
    expect(v).toBe(1 + 27);
    expect(r).toEqual(new U256("0x452361fcdbb033ad9cc8d70a5fb18f72e100fffd7175adfca72a5ea5339ce505"));
    expect(s).toEqual(new U256("0x1294f76e5fbe2cc8fdfb3bcb34253de92678bf7f65646a6dfa58bb0c6d83c839"));
});

test("signed hash", () => {
    const t = new Transaction(new U256(0), new U256(0), new Action("noop"), 1);
    const signed = t.sign(new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd"));
    expect(signed.hash()).toEqual(new H256("20f6fffaf8c3252f683b255dbb8fcf7ffacdb69638b2a9513429fb83342f4a06"));
});
