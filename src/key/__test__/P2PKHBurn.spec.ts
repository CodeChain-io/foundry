import { H256 } from "../../../lib/core/H256";
import { blake256 } from "../../utils";

import { P2PKHBurn } from "../P2PKHBurn";

test("getLockScriptHash()", () => {
    const hash = new H256(blake256(P2PKHBurn.getLockScript()));
    expect(P2PKHBurn.getLockScriptHash()).toEqual(hash);
});
