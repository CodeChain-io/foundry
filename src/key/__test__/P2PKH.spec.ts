import { P2PKH } from "../P2PKH";

test("getLockScriptHash()", () => {
    expect(P2PKH.getLockScriptHash().value).toEqual("f42a65ea518ba236c08b261c34af0521fa3cd1aa505e1c18980919cb8945f8f3");
});
