import { H160 } from "codechain-primitives";

import { blake160 } from "../../utils";

import { P2PKH } from "../P2PKH";

test("getLockScriptHash()", () => {
    const hash = new H160(blake160(P2PKH.getLockScript()));
    expect(P2PKH.getLockScriptHash()).toEqual(hash);
});
