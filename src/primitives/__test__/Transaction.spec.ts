import H256 from "../H256";
import U256 from "../U256";
import Action from "../Action";
import Transaction from "../Transaction";

test("rlp", () => {
    const t = new Transaction(new U256(0), new U256(0), new Action("noop"), 1);
    expect(t.rlpBytes()).toEqual(Buffer.from([0xc4, 0x80, 0x80, 0x80, 0x01]));
});

test("hash", () => {
    const t = new Transaction(new U256(0), new U256(0), new Action("noop"), 1);
    expect(t.hash()).toEqual(new H256("828d02e6a218886c27505cd99cdcc64f22165ce166c4dce331f2dd27b50a4ecf"));
});