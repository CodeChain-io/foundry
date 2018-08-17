import { H256 } from "../../core/H256";
import { blake256 } from "../../utils";

import { P2PKH } from "../P2PKH";

test("getLockScriptHash()", () => {
    const hash = new H256(blake256(P2PKH.getLockScript()));
    expect(P2PKH.getLockScriptHash()).toEqual(hash);
});
